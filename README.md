# WG API
> ♻️ REWRITE IN PROGRESS: [Dioxus](https://dioxuslabs.com) will be replacing React Native as the new Frontend-Framework
> 
> INTO A BRIGHT FUTURE- WITH 🦀🦀 FULLSTACK RUST 🦀🦀
> Peek into the new design: ![image](https://user-images.githubusercontent.com/56208328/232941081-7eb2e8fe-18b6-4316-af9a-66d0a8b05ef6.png)


APi to manage shared costs of your hosehold/community, written in Rust

=> [API Documentation](api/routes.md)

Demo of (Closed Source - React Native) Frontend at: https://wg.kartonrad.de (enter "test", then select a profile and use "test" as a password)

![Screenshot 2023-04-11 185142](https://user-images.githubusercontent.com/56208328/231243492-621f4d36-0a9b-4616-8d75-2d05df87ad0d.png)

![Screenshot 2023-04-11 193054](https://user-images.githubusercontent.com/56208328/231243410-1650eea6-4b28-4b35-835f-8aa1746d51c4.png)

![grafik](https://user-images.githubusercontent.com/56208328/231243910-fabf52a3-1ad2-4b50-8779-0406ff980e28.png)

![grafik](https://user-images.githubusercontent.com/56208328/231243311-c43fca82-1818-451e-8d11-b405a0bf9783.png)

![Screenshot 2023-04-11 135835](https://user-images.githubusercontent.com/56208328/231243620-71e3791e-6f4a-44c0-ba1a-590ccdd625d5.png)

## Usage/Deploy
- Download the binary for your system from the releases tab.
- Download the [.env file](api/.env) from this repositiory and place it into the same folder.

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
