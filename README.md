# Underfloor heating thermostat

Automatically contol a single underfloor heating zone based on
programmed parameters.

## Build / run

In order to use our own partition table, we flash manually with `cargo espflash`.

```
make run
```

Currently a custom partition table is not in use while firmware size
is in flux. Partitioning for OTA updates will be considered later.

## Planned Features

1. Fetch hourly electricity data from a JSON API ([Example data](electricity-price/multiday.json))
2. Increase heating when price is low, to a limit
3. Decrease heating when price is high
4. Turn off heating when price is above a threshold
5. Always turn on heating when temperature drops below a threshold, regardless of price
6. Limits on minimum and maximum heating temperature
7. OLED status display, maybe?
8. OTA firmware updates

## Hardware

Prototype built with ESP32-C6-DevKitC-1.

Later designs will be ESP32-C6-WROOM-1 on a custom PCB.

### Pins

Subject to change:

1. GPIO 10

   Enable current flow through NTC Thermistor for temperature
   measurement.

   Switching current through the thermistor on/off can reduce
   temperature chagne caused by measurement current.

2. GPIO 11

   Switch the underfloor heating relay through a buffer switching FET.

3. I2C pins

   1. Read I2C 12-bit ADC for thermistor voltage (temperature)
   2. Update OLED display (if implemented?)

## Potential options

- Programmable options (i.e. not baked in to the firmware)
