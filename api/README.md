# WG API
APi to manage shared costs of your hosehold/community, written in Rust

=> [API Documentation](routes.md)

## Usage/Deploy
- Download the binary for your system from the releases tab.
- Download the [.env file](.env) from this repositiory and place it into the same folder.

- Set RUST_LOG=info (if you don't want to the logs to get flooded with garbage)
- Edit HOST, and PORT as needed.
- Install PostgresSql, create an empty Database, and a role with Password Authentication that can access it (ALL PRIVILEDGES).
- Set DATABASE_URL to your database url 
- Generate a JWT secret using `openssl rand -base64 32` and set it in JWT_SECRET

- Run the executable (preferably on linux using a systemd service)

- The program will automatically migrate the database, create an `./uploads` and `./uploads/temp` directory, and start up on the specified port

- !!⚠ MAKE SURE to only run wg-api behing NGINX or another reverse proxy, configured to only serve HTTPS with a valid certificate.
- wg-api DOES NOT handle encryption of the connection! 
- IT DOES however, recieve and process CREDENTIALS and other sensitive data
