import fs from 'fs';
import path from 'path';

const packageJsonPath = path.join(process.cwd(), 'package.json');
const tauriConfigPath = path.join(process.cwd(), 'src-tauri', 'tauri.conf.json');

try {
  const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));

  const tauriConfig = JSON.parse(fs.readFileSync(tauriConfigPath, 'utf8'));

  if (tauriConfig.version !== packageJson.version) {
    tauriConfig.version = packageJson.version;

    fs.writeFileSync(tauriConfigPath, JSON.stringify(tauriConfig, null, 2));

    console.log(`Synced version ${packageJson.version} from package.json to tauri.conf.json`);
  } else {
    console.log(`Versions are already in sync: ${packageJson.version}`);
  }
} catch (error) {
  console.error('Error syncing versions:', error);
  process.exit(1);
}
