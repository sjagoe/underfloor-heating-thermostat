# Underfloor heating thermostat

Automatically contol a single underfloor heating zone based on
programmed parameters.

## Features

1. Read hourly electricity price from an influxdb database
2. Increase heating when price is low, to a limit
3. Decrease heating when price is high
4. Turn off heating when price is above a threshold
5. Always turn on heating when temperature drops below a threshold, regardless of price
6. Limits on minimum and maximum heating temperature


## Potential options

- Programmable options (i.e. not baked in to the firmware)
