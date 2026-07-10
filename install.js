const fs = require('fs');
const path = require('path');
const axios = require('axios');
const extract = require('extract-zip');
const { execSync } = require('child_process');

async function downloadAndInstall() {
    const os = process.platform;
    const arch = process.arch;

    let platform = '';
    if (os === 'win32') platform = 'windows';
    else if (os === 'darwin') platform = 'macos';
    else if (os === 'linux') platform = 'linux';
    else {
        console.error(`OS non supporté par NPM: ${os}`);
        process.exit(1);
    }

    if (arch !== 'x64') {
        console.warn(`Attention: Architecture ${arch} détectée. Le binaire risque de ne pas fonctionner, car il est optimisé pour x64.`);
    }

    const binDir = path.join(__dirname, 'bin');
    if (!fs.existsSync(binDir)) {
        fs.mkdirSync(binDir);
    }

    const repo = 'Julien-Bui/skills_pal';
    const fileName = `skills_pal-${platform}-x86_64.zip`;
    const zipPath = path.join(__dirname, fileName);

    try {
        console.log('Recherche de la dernière version sur Github...');
        const releaseRes = await axios.get(`https://api.github.com/repos/${repo}/releases/latest`);
        const version = releaseRes.data.tag_name;
        
        const downloadUrl = `https://github.com/${repo}/releases/download/${version}/${fileName}`;
        console.log(`Téléchargement de Skills Pal ${version} depuis Github...`);
        
        const response = await axios({
            url: downloadUrl,
            method: 'GET',
            responseType: 'stream'
        });

        const writer = fs.createWriteStream(zipPath);
        response.data.pipe(writer);

        await new Promise((resolve, reject) => {
            writer.on('finish', resolve);
            writer.on('error', reject);
        });

        console.log('Extraction du binaire...');
        await extract(zipPath, { dir: binDir });
        fs.unlinkSync(zipPath); // Nettoyer le zip

        // Rendre le fichier exécutable sur Mac/Linux
        if (os !== 'win32') {
            const binPath = path.join(binDir, 'skills_pal');
            execSync(`chmod +x "${binPath}"`);
        }

        console.log('✅ Skills Pal installé avec succès via NPM !');
    } catch (err) {
        console.error('❌ Erreur lors de l\'installation :', err.message);
        process.exit(1);
    }
}

downloadAndInstall();
