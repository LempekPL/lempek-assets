#!/bin/bash
set -e

echo "Creating folders..."
rm -rf ./lempek-assets
mkdir -p ./lempek-assets/frontend
mkdir -p ./lempek-assets/backend

echo "Building backend..."
cd ./backend
cargo build --release

echo "Copying backend binary..."
cp ./Rocket.toml ./example.env ./target/release/lempek-assets-backend ../lempek-assets/backend

echo "Building frontend..."
cd ../frontend
npm i
cat > .env <<'EOF'
BACKEND_URL=https://as.lempek.dev/api
ASSETS_URL=https://a.lempek.dev/
EOF
npm run build

echo "Copying frontend files..."
cp -r ./.output/* ../lempek-assets/frontend
cd ..

echo "Creating runner and init for frontend..."
cat > ./lempek-assets/frontend/start.sh <<'EOF'
#!/bin/bash
export PORT=7002
node ./server/index.mjs
EOF
chmod +x ./lempek-assets/frontend/start.sh

echo "Creating run.sh in main folder..."
cat > ./lempek-assets/run.sh << 'EOF'
#!/bin/bash
set -e

echo "Starting backend..."
cd ./backend
./lempek-assets-backend &
BACKEND_PID=$!
cd ..

echo "Starting frontend..."
cd ./frontend
./start.sh &
FRONTEND_PID=$!
cd ..

echo "Backend PID: $BACKEND_PID"
echo "Frontend PID: $FRONTEND_PID"

echo "Press CTRL+C to stop."

trap "echo 'Stopping...'; kill $BACKEND_PID $FRONTEND_PID; exit 0" SIGINT SIGTERM

wait $BACKEND_PID $FRONTEND_PID
EOF

chmod +x ./lempek-assets/run.sh

echo "Zipping lempek-assets.tar.gz..."
tar -czf lempek-assets.tar.gz lempek-assets

rm -fr ./lempek-assets

echo "Done!"