#!/bin/bash
set -e

# Detect OS and Architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}" in
    Linux*)     PLATFORM="x86_64-unknown-linux-gnu";;
    Darwin*)    PLATFORM="x86_64-apple-darwin";;
    *)          echo "Système d'exploitation non supporté: ${OS}"; exit 1;;
esac

if [ "${ARCH}" = "x86_64" ] || [ "${ARCH}" = "amd64" ]; then
    ARCH="x86_64"
else
    echo "Architecture non supportée: ${ARCH}. Seul x86_64 est supporté pour le moment."
    exit 1
fi

REPO="Julien-Bui/skills_pal"
LATEST_RELEASE=$(curl -s https://api.github.com/repos/${REPO}/releases/latest | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "${LATEST_RELEASE}" ]; then
    echo "Impossible de trouver la dernière release."
    exit 1
fi

echo "🚀 Téléchargement de Skills Pal (${LATEST_RELEASE}) pour ${OS}..."

FILE_NAME="skills_pal-${PLATFORM}.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST_RELEASE}/${FILE_NAME}"

curl -sL -f "${DOWNLOAD_URL}" -o skills_pal.tar.gz || {
    echo "❌ Erreur de téléchargement du binaire."
    exit 1
}

tar -xzf skills_pal.tar.gz

chmod +x skills_pal
chmod +x spal
sudo mv skills_pal /usr/local/bin/
sudo mv spal /usr/local/bin/

rm skills_pal.tar.gz

echo "✅ Skills Pal a été installé avec succès !"
echo "👉 Lance la commande 'skills_pal init' pour commencer."
