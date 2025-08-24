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
npm run build

echo "Copying frontend files..."
cp ./.output/nitro.json ../lempek-assets/frontend
cp -r ./.output/public ../lempek-assets/frontend
# move everyting but the node_modules
mkdir ../lempek-assets/frontend/server
cp -r ./.output/server/chunks ../lempek-assets/frontend/server
cp ./.output/server/index.mjs ../lempek-assets/frontend/server
cp ./.output/server/index.mjs.map ../lempek-assets/frontend/server
cp ./.output/server/package.json ../lempek-assets/frontend/server

cd ../lempek-assets/frontend/server
npm i
cd ../../..

echo "Creating runner and init for frontend..."
cat > ./lempek-assets/frontend/start.sh <<'EOF'
#!/bin/bash

if [ ! -d "node_modules" ]; then
  echo "node_modules not found, installing..."
  cd ./server
  npm i
  cd ..
fi

export PORT=7002
node ./server/index.mjs
EOF
chmod +x ./lempek-assets/frontend/start.sh

echo "Creating run.sh in main folder..."
cat > ./lempek-assets/run.sh << 'EOF'
#!/bin/bash
set -e

echo "Starting backend..."
./backend/lempek-assets-backend &
BACKEND_PID=$!

echo "Starting frontend..."
./frontend/start.sh &
FRONTEND_PID=$!

echo "Backend PID: $BACKEND_PID"
echo "Frontend PID: $FRONTEND_PID"

echo "Press CTRL+C to stop."

trap "echo 'Stopping...'; kill $BACKEND_PID $FRONTEND_PID; exit 0" SIGINT SIGTERM

wait $BACKEND_PID $FRONTEND_PID
EOF

chmod +x ./lempek-assets/run.sh

echo "Zipping lempek-assets.tar.gz..."
tar -czf lempek-assets.tar.gz lempek-assets

echo "Done!"