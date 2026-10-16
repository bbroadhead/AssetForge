use std::{fs, path::{Path, PathBuf}, sync::Mutex};
use tauri::{Manager, State};
use rusqlite::{params, Connection};
use serde::Serialize;
use walkdir::WalkDir;
use base64::{engine::general_purpose::STANDARD, Engine};

struct Db(Mutex<Connection>);
#[derive(Serialize)]
struct Asset { id: i64, path: String, name: String, category: String, extension: String, size: i64, favorite: bool, modified: i64 }

fn category(ext: &str) -> &'static str {
    match ext {
        "png"|"jpg"|"jpeg"|"webp"|"gif"|"bmp"|"svg"|"ase"|"aseprite"|"psd"|"kra" => "Images",
        "wav"|"ogg"|"mp3"|"flac"|"aiff"|"mid"|"midi" => "Audio",
        "fbx"|"obj"|"gltf"|"glb"|"blend"|"stl" => "3D Models",
        "ttf"|"otf"|"woff"|"woff2" => "Fonts",
        "tscn"|"scn"|"unity"|"prefab" => "Scenes",
        "gd"|"cs"|"lua"|"js"|"ts"|"py"|"shader"|"gdshader" => "Scripts",
        "mp4"|"webm"|"mov" => "Video",
        "json"|"xml"|"yaml"|"yml"|"txt"|"md"|"csv" => "Data",
        _ => "Other"
    }
}
fn supported(ext: &str) -> bool { !matches!(category(ext), "Other") || matches!(ext, "zip"|"rar"|"7z"|"pdf") }

#[tauri::command]
fn scan_folder(folder: String, db: State<Db>) -> Result<usize,String> {
    let root=PathBuf::from(&folder);
    if !root.is_dir() { return Err("Choose an existing folder".into()); }
    let root=fs::canonicalize(root).map_err(|e|e.to_string())?;
    let mut conn=db.0.lock().map_err(|e|e.to_string())?;
    let tx=conn.transaction().map_err(|e|e.to_string())?;
    let mut count=0;
    for entry in WalkDir::new(&root).follow_links(false).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() { continue; }
        let path=entry.path();
        let ext=path.extension().and_then(|e|e.to_str()).unwrap_or("").to_ascii_lowercase();
        if !supported(&ext) { continue; }
        let Ok(meta)=entry.metadata() else {continue};
        let name=path.file_name().to_string_lossy().to_string();
        let modified=meta.modified().ok().and_then(|m|m.duration_since(std::time::UNIX_EPOCH).ok()).map(|d|d.as_secs() as i64).unwrap_or(0);
        tx.execute("INSERT INTO assets(path,name,category,extension,size,modified) VALUES (?1,?2,?3,?4,?5,?6) ON CONFLICT(path) DO UPDATE SET name=excluded.name,category=excluded.category,extension=excluded.extension,size=excluded.size,modified=excluded.modified",params![path.to_string_lossy(),name,category(&ext),ext,meta.len() as i64,modified]).map_err(|e|e.to_string())?;
        count+=1;
    }
    tx.execute("INSERT OR IGNORE INTO roots(path) VALUES (?1)",params![root.to_string_lossy()]).map_err(|e|e.to_string())?;
    tx.commit().map_err(|e|e.to_string())?;
    Ok(count)
}
#[tauri::command]
fn list_assets(db: State<Db>) -> Result<Vec<Asset>,String> {
    let conn=db.0.lock().map_err(|e|e.to_string())?;
    let mut stmt=conn.prepare("SELECT id,path,name,category,extension,size,favorite,modified FROM assets ORDER BY name COLLATE NOCASE").map_err(|e|e.to_string())?;
    let rows=stmt.query_map([],|r|Ok(Asset{id:r.get(0)?,path:r.get(1)?,name:r.get(2)?,category:r.get(3)?,extension:r.get(4)?,size:r.get(5)?,favorite:r.get::<_,i64>(6)?!=0,modified:r.get(7)?})).map_err(|e|e.to_string())?;
    rows.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())
}
#[tauri::command]
fn toggle_favorite(id: i64, db: State<Db>) -> Result<(),String> {
    db.0.lock().map_err(|e|e.to_string())?.execute("UPDATE assets SET favorite=1-favorite WHERE id=?1",params![id]).map_err(|e|e.to_string())?;Ok(())
}
#[tauri::command]
fn remove_asset(id: i64, db: State<Db>) -> Result<(),String> {
    db.0.lock().map_err(|e|e.to_string())?.execute("DELETE FROM assets WHERE id=?1",params![id]).map_err(|e|e.to_string())?;Ok(())
}
#[tauri::command]
fn preview_image(path: String, db: State<Db>) -> Result<Option<String>,String> {
    let conn=db.0.lock().map_err(|e|e.to_string())?;
    let exists: i64=conn.query_row("SELECT COUNT(*) FROM assets WHERE path=?1",params![path],|r|r.get(0)).map_err(|e|e.to_string())?;
    if exists==0 {return Err("Asset is not in the library".into());}
    let ext=Path::new(&path).extension().and_then(|x|x.to_str()).unwrap_or("").to_ascii_lowercase();
    let mime=match ext.as_str(){"png"=>"image/png","jpg"|"jpeg"=>"image/jpeg","gif"=>"image/gif","webp"=>"image/webp","bmp"=>"image/bmp",_=>return Ok(None)};
    let meta=fs::metadata(&path).map_err(|e|e.to_string())?;
    if meta.len()>8*1024*1024 {return Ok(None)};
    let bytes=fs::read(&path).map_err(|e|e.to_string())?;
    Ok(Some(format!("data:{};base64,{}",mime,STANDARD.encode(bytes))))
}
pub fn run(){
    tauri::Builder::default().plugin(tauri_plugin_dialog::init()).setup(|app|{
        let dir=app.path().app_data_dir()?;fs::create_dir_all(&dir)?;
        let conn=Connection::open(dir.join("assetforge.sqlite"))?;
        conn.execute_batch("PRAGMA journal_mode=WAL; CREATE TABLE IF NOT EXISTS assets(id INTEGER PRIMARY KEY,path TEXT NOT NULL UNIQUE,name TEXT NOT NULL,category TEXT NOT NULL,extension TEXT NOT NULL,size INTEGER NOT NULL,modified INTEGER NOT NULL,favorite INTEGER NOT NULL DEFAULT 0); CREATE TABLE IF NOT EXISTS roots(path TEXT PRIMARY KEY);")?;
        app.manage(Db(Mutex::new(conn)));Ok(())
    }).invoke_handler(tauri::generate_handler![scan_folder,list_assets,toggle_favorite,remove_asset,preview_image]).run(tauri::generate_context!()).expect("error running AssetForge");
}