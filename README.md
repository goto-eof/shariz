```
   ______            _   
  / __/ /  ___ _____(_)__
 _\ \/ _ \/ _ `/ __/ /_ /
/___/_//_/\_,_/_/ /_//__/
                         
```

# What is Shariz?
Shariz, like dropbox, is a file sharing application. Shariz was implemented in Rust. For now it allows to share files between 2 computers.

# How it works?
At startup Shariz loads the target server ip and port from the configuration file and creates a client that will connect to the target server. Meanwhile it makes a connection with the target application, requests for files and downloads them.

![shariz flow](images/flow.png)

# How to configure it?

![shariz configuration](images/configuration.png)

![shariz configuration](images/dot_env.png)

On the first PC replace the `target_ip` with the others PC connected on the same network. Update also the `shared_directory` property that should point to the directory that will be shared. Do the same on other PC. Add/remove some file from the shared directory of the first PC and magically you will see that the same operation is done also in the shared directory of the second PC.

# How to configure and run the standalone?

![shariz configuration](images/struct.png)

 First of all download the latest executable compatible with your OS (Windows, MacOS, Linux) from [here](https://github.com/goto-eof/shariz/releases). Then create the .env file and the configuration.json file like was done in the previous screenshot. Edit the configuration.json as described in the previous paragraph. Run shariz from command line ./shariz. Do the same for the second PC. 

# Screenshot

![shariz flow](images/screenshot.png)

# Development stack

![shariz flow](images/dev_stack.png)

# For developers
```
sudo apt-get install libsqlite3-dev
```

```
cargo install diesel_cli --no-default-features --features sqlite

diesel migration run
```