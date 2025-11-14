# Building asset server with build.sh script

## Requirements
- Rust toolchain installed (cargo)
- Node.js and npm installed
- systemd-enabled Linux server
- nginx configured for your domains

## Quick Start
1. Clone the project with `git clone https://github.com/LempekPL/lempek-assets.git`
2. Edit values in main.env (see Configuring `.env` section below)
3. Run `cd lempek-assets/build`
4. Run the script `./build.sh [--no-tar]`\
Use the optional --no-tar flag to skip archive generation.
5. Wait for the build to complete.
6. Move the generated archive (lempek-assets.tar.gz) to your deployment location.
7. Extract the archive and run `./setup.sh`

## Configuring `.env`

- DATABASE_URL\
this uses standard postgres url connection syntax\
`postgres://admin:password@localhost:5432/database`\
the database itself must already exist to work properly


- FILES_DIR\
Directory where files will be physically stored


- JWT_SECRET\
Random string of characters


- SECRET_KEY\
May either be a 256-bit base64 or hex string or a slice of 32 bytes\
You can use `openssl rand -base64 32` to generate it


- BACKEND_PORT\
Number, your port for backend to use internally


- FRONTEND_PORT\
Number, your port for frontend to use internally


- PUBLIC_ASSETS_URL\
URL with the domain on where the assets will be hosted


- PUBLIC_BACKEND_URL\
URL with the domain on where the api will be hosted


- PUBLIC_FRONTEND_URL\
  URL with the domain on where the website will be hosted


