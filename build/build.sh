#!/bin/bash
set -e

cd "$(dirname "$0")"

NO_TAR=false
if [[ "$1" == "--no-tar" ]]; then
  NO_TAR=true
fi

set -a
source ./main.env
set +a

cd ..
WORKDIR="lempek-assets"
FRONTDIR="$WORKDIR/frontend"
BACKDIR="$WORKDIR/backend"

echo "Creating folders..."
rm -rf "$WORKDIR"
mkdir -p "$FRONTDIR" "$BACKDIR"

echo "Building backend..."
cd ./backend
cargo build --release

echo "Copying backend binary..."
cat > .env <<EOF
DATABASE_URL=${DATABASE_URL}
JWT_SECRET=${JWT_SECRET}
FILES_DIR=${FILES_DIR}
ALLOWED_ORIGIN=${PUBLIC_FRONTEND_URL}
ROCKET_PORT=${BACKEND_PORT}
ROCKET_SECRET_KEY=${SECRET_KEY}
EOF
cp ./Rocket.toml ./target/release/lempek-assets-backend .env "../$BACKDIR"
rm .env
cd ..

echo "Building frontend..."
cd ./frontend
npm i
cat > .env <<'EOF'
BACKEND_URL=${PUBLIC_BACKEND_URL}
ASSETS_URL=${PUBLIC_ASSETS_URL}
EOF
npm run build

echo "Copying frontend files..."
cp .env "../$FRONTDIR"
cp -r ./.output/* ../lempek-assets/frontend
cd ..

echo "Creating frontend starter..."
cat > "$FRONTDIR/start.sh" <<'EOF'
#!/bin/bash
export PORT=${FRONTEND_PORT}
node ./server/index.mjs
EOF
chmod +x "$FRONTDIR/start.sh"

echo "Creating systemd setup script..."
cat > "$WORKDIR/setup.sh" <<'EOF'
#!/bin/bash
set -e

# Backend systemd
sudo tee /etc/systemd/system/lempek-assets-back.service > /dev/null <<SERVICE
[Unit]
Description=Lempek Assets Backend
After=network.target

[Service]
Type=simple
WorkingDirectory=$(pwd)/backend
ExecStart=$(pwd)/backend/lempek-assets-backend
EnvironmentFile=$(pwd)/backend/.env
User=$USER
Restart=on-failure

[Install]
WantedBy=multi-user.target
SERVICE

# Frontend systemd
sudo tee /etc/systemd/system/lempek-assets-front.service > /dev/null <<SERVICE
[Unit]
Description=Lempek Assets Frontend
After=network.target

[Service]
Type=simple
WorkingDirectory=$(pwd)/frontend
ExecStart=node ./server/index.mjs
User=$USER
Restart=on-failure

[Install]
WantedBy=multi-user.target
SERVICE

sudo systemctl daemon-reload
sudo systemctl enable lempek-assets-back.service
sudo systemctl enable lempek-assets-front.service

echo "Systemd services configured! Use 'sudo systemctl start lempek-assets-back lempek-assets-front' to launch them."
EOF
chmod +x "$WORKDIR/setup.sh"

if [ "$NO_TAR" = false ]; then
  echo "Packing $WORKDIR into tar.gz..."
  tar -czf lempek-assets.tar.gz "$WORKDIR"
  rm -rf "$WORKDIR"
else
  echo "Skipping tar.gz creation as --no-tar option is set."
fi

rm -rf "$WORKDIR"
echo "Done!"