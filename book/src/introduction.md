# Introduction

This server allows you to integrate an ESC/POS compatible receipt printer via MQTT.
It translates programs written in a DSL an received via MQTT into ESC/POS programs, and forwards these programs to a printer.

## Connection to the printer
Currently only networked ESC/POS printers are supported.
They are auto-discovered via network.
A log message will indicate which printers were found.
Discovery is done using Epsons discovery protocol which records responses to a UDP multicast packet.
The printer is then identified via SNMP.

Connections to all printers are initiated on port 9100 (RAW printing port).

You also have the option to manually configure a printers network settings.
To do so, specify the hostname or IP address in the `MANUAL_PRINTER_HOST` environment variable.
You also have the option to specify the printer model for that printer in the `MANUAL_PRINTER_MODEL` variable.

The default fallback model (if it cannot be discovered for example) can be overriden using the `DEFAULT_PRINTER_MODEL` variable.

See a list of supported printer model values [in the documentation of escpos-db](https://docs.rs/escpos-db/0.1.2/src/escpos_db/gen.rs.html#2235).

## Connection to MQTT
Configure the connection to your MQTT broker using the `MQTT_URL` variable.
Example values for that may be:

```
mqtt://username:password@mqtt-broker/
mqtt://broker/
```

## Printing
To print, send a program to the printers MQTT topic.
A program is a newline-separated listed of commands.
See the [command reference](command-reference.md) for a description of all commands.

The specific topic of a printer follows the structure `escpos/{printer_id}/print`, where `printer_id` is the printers ID.
Find the printers ID by checking the logs, or `manual` if you want to use the manual printer.
