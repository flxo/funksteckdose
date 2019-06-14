#!/bin/bash

set -x

scp $1 pi@raspberrypi:/tmp/funksteckdose
ssh pi@raspberrypi sudo RUST_LOG=funksteckdose=debug /tmp/funksteckdose -g 10011 -d 10000 -s on
sleep 1
ssh pi@raspberrypi sudo RUST_LOG=funksteckdose=debug /tmp/funksteckdose -g 10011 -d 10000 -s off