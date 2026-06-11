use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use ed25519_dalek::{SigningKey, VerifyingKey, Signer, Verifier, Signature};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use std::io::{Write as IoWrite, Read as IoRead};
use crate::models::Template;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateManifest {
    pub version: String,
    pub template: Template,
    pub signature: Option<String>,
    pub signed_by: Option<String>,
    pub public_key: Option<String>,
    pub checksum: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPair {
    pub public_key: String,
    pub private_key: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureVerification {
    pub is_valid: bool,
    pub signed_by: Option<String>,
    pub checksum_valid: bool,
    pub error: Option<String>,
}

pub fn generate_keypair(name: &str) -> Result<KeyPair, String> {
    let mut csprng = rand::rngs::OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();

    let public_key_bytes = verifying_key.to_bytes();
    let private_key_bytes = signing_key.to_bytes();

    Ok(KeyPair {
        public_key: BASE64.encode(public_key_bytes),
        private_key: BASE64.encode(private_key_bytes),
        name: name.to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    })
}

pub fn compute_template_checksum(template: &Template) -> Result<String, String> {
    let yaml = template.to_yaml()?;
    let mut hasher = Sha256::new();
    hasher.update(yaml.as_bytes());
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

pub fn sign_template(
    template: &Template,
    private_key: &str,
    signer_name: &str,
) -> Result<TemplateManifest, String> {
    let checksum = compute_template_checksum(template)?;

    let private_key_bytes = BASE64
        .decode(private_key)
        .map_err(|e| format!("无效的私钥编码: {}", e))?;
    let signing_key = SigningKey::from_bytes(
        private_key_bytes.as_slice().try_into()
            .map_err(|_| "无效的私钥长度".to_string())?
    );
    let verifying_key = signing_key.verifying_key();

    // Sign the checksum
    let message = checksum.as_bytes();
    let signature = signing_key.sign(message);
    let signature_bytes = signature.to_bytes();

    Ok(TemplateManifest {
        version: "1.0".to_string(),
        template: template.clone(),
        signature: Some(BASE64.encode(signature_bytes)),
        signed_by: Some(signer_name.to_string()),
        public_key: Some(BASE64.encode(verifying_key.to_bytes())),
        checksum,
        created_at: chrono::Utc::now().to_rfc3339(),
    })
}

pub fn verify_template(manifest: &TemplateManifest) -> Result<SignatureVerification, String> {
    // Verify checksum
    let current_checksum = compute_template_checksum(&manifest.template)?;
    let checksum_valid = current_checksum == manifest.checksum;

    // If no signature, return checksum-only result
    let signature = match &manifest.signature {
        Some(sig) => sig,
        None => {
            return Ok(SignatureVerification {
                is_valid: false,
                signed_by: manifest.signed_by.clone(),
                checksum_valid,
                error: Some("模板未签名".to_string()),
            });
        }
    };

    let public_key_str = match &manifest.public_key {
        Some(pk) => pk,
        None => {
            return Ok(SignatureVerification {
                is_valid: false,
                signed_by: manifest.signed_by.clone(),
                checksum_valid,
                error: Some("缺少公钥".to_string()),
            });
        }
    };

    let public_key_bytes = BASE64
        .decode(public_key_str)
        .map_err(|e| format!("无效的公钥编码: {}", e))?;
    let verifying_key = VerifyingKey::from_bytes(
        public_key_bytes.as_slice().try_into()
            .map_err(|_| "无效的公钥长度".to_string())?
    ).map_err(|e| format!("无效的公钥: {}", e))?;

    let signature_bytes = BASE64
        .decode(signature)
        .map_err(|e| format!("无效的签名编码: {}", e))?;
    let sig = Signature::from_bytes(
        signature_bytes.as_slice().try_into()
            .map_err(|_| "无效的签名长度".to_string())?
    );

    let message = manifest.checksum.as_bytes();
    let is_valid = verifying_key.verify(message, &sig).is_ok();

    Ok(SignatureVerification {
        is_valid,
        signed_by: manifest.signed_by.clone(),
        checksum_valid,
        error: if is_valid && checksum_valid {
            None
        } else if !checksum_valid {
            Some("校验和不匹配，模板内容可能被篡改".to_string())
        } else {
            Some("签名验证失败".to_string())
        },
    })
}

pub fn export_template(
    template: &Template,
    private_key: Option<&str>,
    signer_name: Option<&str>,
) -> Result<Vec<u8>, String> {
    let manifest = match (private_key, signer_name) {
        (Some(pk), Some(name)) => sign_template(template, pk, name)?,
        _ => {
            let checksum = compute_template_checksum(template)?;
            TemplateManifest {
                version: "1.0".to_string(),
                template: template.clone(),
                signature: None,
                signed_by: None,
                public_key: None,
                checksum,
                created_at: chrono::Utc::now().to_rfc3339(),
            }
        }
    };

    let manifest_yaml = serde_yaml::to_string(&manifest)
        .map_err(|e| format!("序列化清单失败: {}", e))?;
    let template_yaml = template.to_yaml()?;

    // Create ZIP archive
    let buf = std::io::Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(buf);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    zip.start_file("manifest.yml", options)
        .map_err(|e| format!("创建ZIP条目失败: {}", e))?;
    zip.write_all(manifest_yaml.as_bytes())
        .map_err(|e| format!("写入清单失败: {}", e))?;

    zip.start_file("template.yml", options)
        .map_err(|e| format!("创建ZIP条目失败: {}", e))?;
    zip.write_all(template_yaml.as_bytes())
        .map_err(|e| format!("写入模板失败: {}", e))?;

    let result = zip.finish()
        .map_err(|e| format!("完成ZIP压缩失败: {}", e))?;

    Ok(result.into_inner())
}

pub fn import_template_from_file(file_path: &str) -> Result<TemplateManifest, String> {
    let file = std::fs::File::open(file_path)
        .map_err(|e| format!("打开文件失败: {}", e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("读取ZIP文件失败: {}", e))?;

    // Read manifest.yml
    let manifest_yaml = {
        let mut manifest_file = archive.by_name("manifest.yml")
            .map_err(|e| format!("读取manifest.yml失败: {}", e))?;
        let mut content = String::new();
        manifest_file.read_to_string(&mut content)
            .map_err(|e| format!("读取清单内容失败: {}", e))?;
        content
    };

    let manifest: TemplateManifest = serde_yaml::from_str(&manifest_yaml)
        .map_err(|e| format!("解析清单失败: {}", e))?;

    // Verify checksum against template.yml content
    let template_yaml = {
        let mut template_file = archive.by_name("template.yml")
            .map_err(|e| format!("读取template.yml失败: {}", e))?;
        let mut content = String::new();
        template_file.read_to_string(&mut content)
            .map_err(|e| format!("读取模板内容失败: {}", e))?;
        content
    };

    let mut hasher = Sha256::new();
    hasher.update(template_yaml.as_bytes());
    let computed_checksum = format!("{:x}", hasher.finalize());

    if computed_checksum != manifest.checksum {
        return Err(format!(
            "校验和不匹配！文件可能已损坏或被篡改 (期望: {}, 实际: {})",
            manifest.checksum, computed_checksum
        ));
    }

    Ok(manifest)
}
