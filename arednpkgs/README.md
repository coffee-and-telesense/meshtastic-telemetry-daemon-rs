# Packages for Mikrotik hAP ac lite

This folder contains packages that enable serial drivers on the USB port of the Mikrotik hAP ac lite when it is running AREDN 3.24.10.0

I have them here for convenience at the moment.

When publishing this repo, and removing its private status, perhaps we change this.

In order to install them: 

1. Login to the AREDN web interface
2. Go to the administration/setup area
3. Advanced is the subpage where you can upload and install packages
4. Install them in this order: `kmod-nls-base` -> `kmod-usb-core` -> `kmod-phy-ath79-usb` -> `kmod-usb-ehci` -> `kmod-usb-acm`

Now after installing them all, and several reboots later, you can plug in a serial device to the USB port. You should see `/dev/ttyACM0` available now

