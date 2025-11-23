# Core Modules Vs Wasteland Modules

**Simply put**, core modules help you interact with the Survon OS, and wasteland modules help you interact with your homestead.

**More deeply put**, Survon comes with some built-in functionality. Core modules represent a way for you to interface with that functionality so 
the operating system can offer you some value from day one. Because we built them, they offer a lot more functionality than 
wasteland modules. They're much more like mini-apps: to-do list, calendar, knowledge base chat interface, etc. 

Wasteland modules are a way for Survon to represent data from, and interface with, external hardware systems. The Survon system 
captures various inbound data from serial/radio/bluetooth/etc. It parses the packets and directs Survon-compatible event messages
to the central message bus for parsing. The inbound data is redirected appropriately to its corresponding wasteland module so it can 
be displayed on the TUI. Each registered piece of hardware must have a corresponding wasteland module so we can both display its data 
as well as communicate back to it over the same payload contract. These wasteland modules assume that the IoT device in play has 
been configured to adhere to the Survon serial bus contract. 

