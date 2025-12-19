import fs from 'fs';
import path from 'path';

// Get version from command line arguments
const newVersion = process.argv[2];

if (!newVersion) {
  console.error('Error: No version provided. Usage: node sync-versions.js <version>');
  process.exit(1);
}

const packageJsonPath = path.join(process.cwd(), 'package.json');
const tauriConfigPath = path.join(process.cwd(), 'src-tauri', 'tauri.conf.json');
const cargoTomlPath = path.join(process.cwd(), 'src-tauri', 'Cargo.toml');

try {
  // Update package.json
  const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));
  packageJson.version = newVersion;
  fs.writeFileSync(packageJsonPath, JSON.stringify(packageJson, null, 2));
  console.log(`Updated version in package.json to ${newVersion}`);

  // Update tauri.conf.json
  const tauriConfig = JSON.parse(fs.readFileSync(tauriConfigPath, 'utf8'));
  tauriConfig.version = newVersion;
  fs.writeFileSync(tauriConfigPath, JSON.stringify(tauriConfig, null, 2));
  console.log(`Updated version in tauri.conf.json to ${newVersion}`);

  // Update Cargo.toml
  let cargoTomlContent = fs.readFileSync(cargoTomlPath, 'utf8');
  cargoTomlContent = cargoTomlContent.replace(/^(version = )"[^"]+"/m, `$1"${newVersion}"`);
  fs.writeFileSync(cargoTomlPath, cargoTomlContent);
  console.log(`Updated version in Cargo.toml to ${newVersion}`);
} catch (error) {
  console.error('Error syncing versions:', error);
  process.exit(1);
}
