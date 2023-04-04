```
   ______            _   
  / __/ /  ___ _____(_)__
 _\ \/ _ \/ _ `/ __/ /_ /
/___/_//_/\_,_/_/ /_//__/
                         
```
# Shariz is a Work In Progress project

# What is Shariz?
Shariz, like dropbox, is a file sharing application implemented in Rust. For now it will allow to share files between 2 computers on the same network.

# How it works?
At startup Shariz loads the target server ip and port from the configuration file and creates a client that will connect to the target server. Menwhile it scans the directory configured in the configuration file and searches for files. If there are new files, then Shariz will send those files to the target server. Shariz acts as server and as client.

# Screenshot
no screenshot