#!/bin/bash

# Install script for RPi4s to setup the mesh telemetry user service for 915 and 433 deployments
mv mesh-telem433.service /etc/systemd/system/mesh-telem433.service
mv mesh-telem915.service /etc/systemd/system/mesh-telem915.service
mv 92-usb-input-no-powersave.rules /etc/udev/rules.d/92-usb-input-no-powersave.rules

# Reload systemd services
systemctl daemon-reload

# Reload udev rules
udevadm control --reload-rules && udevadm trigger
