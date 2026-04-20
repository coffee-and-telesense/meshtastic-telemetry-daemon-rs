#!/bin/sh

# Install script for RPi4s to setup the mesh telemetry user service for 915 and 433 deployments
mv mesh-telem.service /etc/systemd/system/mesh-telem.service
mv 92-usb-input-no-powersave.rules /etc/udev/rules.d/92-usb-input-no-powersave.rules

# Reload systemd services
systemctl daemon-reload

# Reload udev rules
udevadm control --reload-rules && udevadm trigger
