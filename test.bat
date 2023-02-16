@echo off
echo =======Building binary=======
cargo build --quiet
echo =======Testing=======
:: uhhhhhhhhhhhhhhhhhhhhhhhhhhh test? framework? 
cargo mommy test   
echo =======Cleaning up the child process=======
taskkill /IM wg-api.exe /F