```
   ______            _   
  / __/ /  ___ _____(_)__
 _\ \/ _ \/ _ `/ __/ /_ /
/___/_//_/\_,_/_/ /_//__/
                         
```
# Shariz is a Work In Progress project

# What is Shariz?
Shariz, like dropbox, is a file sharing application. Shariz was implemented in Rust. For now it will allow to share files between 2 computers on the same network.

# How it works?
At startup Shariz loads the target server ip and port from the configuration file and creates a client that will connect to the target server. Menwhile it makes a connection with the target application, requests for files and download them.
!(shariz flow)[flow.png]

# Screenshot
no screenshot
