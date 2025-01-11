#!/bin/bash

# Check which "alias command" was passed
case "$1" in
    stm|stm32|car_stm32/)
        cd ./car_stm32 && cargo embed --release
        ;;
    esp|esp32|car_esp32/)
        cd ./car_esp32 && cargo run --release
        ;;
    remote|remote_esp|remote_esp32/)
        cd ./remote_esp32 && cargo run --release
        ;;
    *)
        echo "Unknown command. Available commands:"
        echo "  stm         - Embed stm and open serial monitor"
        echo "  esp         - Embed car esp32 camera and connect"
        echo "  remote      - Embed remote esp32 and connect"
        ;;
esac