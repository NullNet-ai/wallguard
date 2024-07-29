# WallMon

**The problem:**

Firewalls protect networks but each installation creates a risk profile, mainly due to misconfigurations. This risk increases exponentially with more firewalls, especially when managed by multiple administrators across various locations.

**Solution:**

Centralize firewall configuration management through a central interface with built-in control and approval mechanisms. The system will:

•	Check configurations against a set of rules to warn or prevent misconfigurations.
•	Include firewall traffic monitoring.
•	Incorporate proactive IDS and IDP using new AI models.

**Approach:**

1. Starting Point:
   Use pfSense, a widely used and familiar firewall.
2. Installation:
   Deploy a minimal dependency C application on pfSense to send configuration files to a central serve
3. Central Management Interface:
   Parse and display configurations on a centralized interface.
   Use a diagramming tool to visualize the network layout of firewalls.
   View individual or collective firewall configurations.
   Visualize effective traffic flow based on the network diagram.
4. Additional Features:
   Include live traffic flow visualization.
   Implement configuration approval processes.
   Integrate IDP into the management interface.

This centralized system will streamline firewall management, reduce risks, and improve network security.

The purpose of this repository is to monitor firewall configuration and upload to the central server.

Once a change is made the program downloads that file and upon receiving a command, it will apply the configuration.

**Implementation:**

---

Design philosopy:

* Zero to minimum dependency.
* Least amount of processing.
* Data corruption prevention.
* Prioritize backup and restore.

**Common functions:**

* Validate Firewall: Validate that the firewall is the correct make, model, version and all necessary files necessary are present.
* Get unique id of the machine: To uniquely identify the installation of the unit and detect any cloning of the machine.
* Validate machine id: Upon startup look for the machine id file in the config directory. If it does not exist, get the DMI UUID of the machine and set working ID to the value. Save the UUID as a file with 0_UnixTimestamp_UUID. 0 indicates the latest and active machine ID. If we can't get the UUID from DMI, raise an exception report it to the central server and stop the program. If the file does exist save the UUID to a variable. Get the DMI UUID and set to working copy variable. Compare the working to saved and if  different remove the 0_ from the file starting with 0_. Save the new UUID as 0_.
* Watch configuration: Watch for any changes to the configuration file and notify the program.
* Get config file date and time: Get the modify date of the configuration file.
* Get config file version: Get the revision, date and time as recorded by the firewall software in the config file.
* Create directory: Create directory structure needed to store state and backup of files.
* Copy file: Copy files and overwrite destination file.
* Generate boundry: Generate a unique boundry string for each request sent to the web server when uploading the config file.
* Send file: Send config file to the central server.
* Receive file: Receive config file from the central server.
* Get request: Make a get request to a web server.
* Post request: Make a post request to a web server.
* Execute command: Execute a shell command and get the response.
