const crypto = require('crypto');
const fs = require('fs');
const path = require('path');

function generateSigningKeys() {
  console.log('Generating Tauri update signing keys...\n');

  const { publicKey, privateKey } = crypto.generateKeyPairSync('ed25519', {
    publicKeyEncoding: { type: 'spki', format: 'pem' },
    privateKeyEncoding: { type: 'pkcs8', format: 'pem' }
  });

  const keyDir = path.join(__dirname, '..', 'keys');
  if (!fs.existsSync(keyDir)) {
    fs.mkdirSync(keyDir, { recursive: true });
  }

  fs.writeFileSync(path.join(keyDir, 'private.key'), privateKey);
  fs.writeFileSync(path.join(keyDir, 'public.key'), publicKey);

  console.log('Keys generated successfully!');
  console.log(`  Private key: ${path.join(keyDir, 'private.key')}`);
  console.log(`  Public key:  ${path.join(keyDir, 'public.key')}`);
  console.log('\nIMPORTANT:');
  console.log('  - Keep private.key SECRET. Never commit it to the repository.');
  console.log('  - Add keys/ to .gitignore');
  console.log('  - Set TAURI_SIGNING_PRIVATE_KEY env var in CI with the private key content');
  console.log('  - Add the public key to tauri.conf.json > plugins > updater > pubkey');
  console.log('\nAlternatively, use the Tauri CLI:');
  console.log('  npm run tauri signer generate -w ~/.tauri/godotharbor.key');
}

function signFile(filePath) {
  if (!filePath) {
    console.error('Please provide a file path to sign');
    process.exit(1);
  }

  const keyPath = path.join(__dirname, '..', 'keys', 'private.key');
  if (!fs.existsSync(keyPath)) {
    console.error('Private key not found. Run "node sign-update.js generate" first.');
    process.exit(1);
  }

  const privateKey = fs.readFileSync(keyPath, 'utf8');
  const data = fs.readFileSync(filePath);

  const sign = crypto.createSign('SHA256');
  sign.update(data);
  sign.end();

  const signature = sign.sign(privateKey, 'base64');

  const sigPath = filePath + '.sig';
  fs.writeFileSync(sigPath, signature);

  console.log(`Signature written to ${sigPath}`);
  console.log(`  Signature: ${signature.substring(0, 40)}...`);
}

const args = process.argv.slice(2);
const command = args[0];

if (command === 'generate') {
  generateSigningKeys();
} else if (command === 'sign') {
  signFile(args[1]);
} else {
  console.log('Usage:');
  console.log('  node sign-update.js generate    - Generate signing key pair');
  console.log('  node sign-update.js sign <file>  - Sign a file with the private key');
}
