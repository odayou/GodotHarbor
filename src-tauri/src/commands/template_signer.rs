use base64::Engine;
use tauri::AppHandle;
use crate::template_signer::{self, KeyPair, TemplateManifest, SignatureVerification};
use crate::commands::utils::{get_data_dir, get_storage};
use crate::storage::Storage;

fn get_keypairs(storage: &Storage) -> Vec<KeyPair> {
    storage.load_or_default("keypairs.json")
}

fn save_keypairs_to_storage(storage: &Storage, keypairs: &Vec<KeyPair>) -> Result<(), String> {
    storage.save("keypairs.json", keypairs)
        .map_err(|e| format!("保存密钥对失败: {}", e))
}

#[tauri::command]
pub fn generate_signing_keypair(name: String) -> Result<KeyPair, String> {
    template_signer::generate_keypair(&name)
}

/// Generate the export data (ZIP bytes as base64). The frontend handles the save dialog.
#[tauri::command]
pub async fn export_template_signed(
    app: AppHandle,
    template_id: String,
    signer_name: Option<String>,
) -> Result<String, String> {
    let app_clone = app.clone();
    let template_id_clone = template_id.clone();
    tokio::task::spawn_blocking(move || {
        // Load template
        let data_dir = get_data_dir(&app_clone);
        let templates_dir = data_dir.join("templates");
        let template_dir = templates_dir.join(&template_id_clone);
        let template_file = template_dir.join("template.yml");

        if !template_file.exists() {
            return Err(format!("模板不存在: {}", template_id_clone));
        }

        let content = std::fs::read_to_string(&template_file)
            .map_err(|e| format!("读取模板文件失败: {}", e))?;
        let template = crate::models::Template::from_yaml(&content)?;

        // Find keypair if signer_name provided
        let private_key = if let Some(ref name) = signer_name {
            let storage = get_storage(&app_clone);
            let keypairs = get_keypairs(&storage);
            let kp = keypairs.iter().find(|k| k.name == *name)
                .ok_or_else(|| format!("未找到签名者密钥: {}", name))?;
            Some(kp.private_key.clone())
        } else {
            None
        };

        // Export
        let data = template_signer::export_template(
            &template,
            private_key.as_deref(),
            signer_name.as_deref(),
        )?;

        // Return base64-encoded data
        Ok(base64::engine::general_purpose::STANDARD.encode(&data))
    }).await.map_err(|e| format!("任务执行失败: {}", e))?
}

/// Write exported template data to a file path chosen by the user (via frontend dialog)
#[tauri::command]
pub fn write_template_export(file_path: String, data_base64: String) -> Result<(), String> {
    let data = base64::engine::general_purpose::STANDARD
        .decode(&data_base64)
        .map_err(|e| format!("解码导出数据失败: {}", e))?;
    std::fs::write(&file_path, &data)
        .map_err(|e| format!("写入文件失败: {}", e))
}

#[tauri::command]
pub async fn import_template_from_file(
    app: AppHandle,
    file_path: String,
) -> Result<TemplateManifest, String> {
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let manifest = template_signer::import_template_from_file(&file_path)?;

        // Save the imported template
        let mut template = manifest.template.clone();
        template.template_id = uuid::Uuid::new_v4().to_string();
        template.is_builtin = false;
        template.source_url = file_path.clone();

        let data_dir = get_data_dir(&app_clone);
        let templates_dir = data_dir.join("templates");
        let template_dir = templates_dir.join(&template.template_id);
        std::fs::create_dir_all(&template_dir)
            .map_err(|e| format!("创建模板目录失败: {}", e))?;

        let template_file = template_dir.join("template.yml");
        let yaml = template.to_yaml()?;
        std::fs::write(&template_file, yaml)
            .map_err(|e| format!("写入模板文件失败: {}", e))?;

        // Return manifest with updated template
        Ok(TemplateManifest {
            template,
            ..manifest
        })
    }).await.map_err(|e| format!("任务执行失败: {}", e))?
}

#[tauri::command]
pub fn verify_template_signature(manifest: TemplateManifest) -> Result<SignatureVerification, String> {
    template_signer::verify_template(&manifest)
}

#[tauri::command]
pub fn get_stored_keypairs(app: AppHandle) -> Result<Vec<KeyPair>, String> {
    let storage = get_storage(&app);
    Ok(get_keypairs(&storage))
}

#[tauri::command]
pub fn save_keypair(app: AppHandle, keypair: KeyPair) -> Result<(), String> {
    let storage = get_storage(&app);
    let mut keypairs = get_keypairs(&storage);
    // Check for duplicate public key
    if keypairs.iter().any(|k| k.public_key == keypair.public_key) {
        return Err("公钥已存在".to_string());
    }
    keypairs.push(keypair);
    save_keypairs_to_storage(&storage, &keypairs)
}

#[tauri::command]
pub fn delete_keypair(app: AppHandle, public_key: String) -> Result<(), String> {
    let storage = get_storage(&app);
    let mut keypairs = get_keypairs(&storage);
    let initial_len = keypairs.len();
    keypairs.retain(|k| k.public_key != public_key);
    if keypairs.len() == initial_len {
        return Err("密钥对不存在".to_string());
    }
    save_keypairs_to_storage(&storage, &keypairs)
}
