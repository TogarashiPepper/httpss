# Background
If HTTPS is better than HTTP because of the extra S, then logically the protocol better than HTTPS would be HTTPSS. This simple project implements just that.

# Usage
Run the following commands to generate your CA and certificates (we'll be self signing here):
```bash
chmod +x ./init.sh
./init.sh
```
Next, get the server started by running 
```bash
cargo run --bin server
```
and if everything went well you'll see an output like `HTTPSS Server listening on 127.0.0.1:4433`, then to actually send an HTTPSS request to your new server, run `cargo run --bin client -- <Your message goes here>` in another tab and you'll see an output like:
```
Response Status: 200 OK
Response Body: b"<Your message>"
```
