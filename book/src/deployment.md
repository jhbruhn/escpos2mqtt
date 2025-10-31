# Deployment

This is a long running service and should be deployed as such.

It is recommend to use a containerized solution for that.
Container images are made available in this repository and are available via the GitHub Container Registry.
Right now there is only one tag which is the most recent version of the `main` branch:

```
ghcr.io/jhbruhn/escpos2mqtt:main
```

To enable UDP-broadcast-based discovery of printers in your network, use host networking to give the service access to the hosts interfaces which sends out the discovery messages and receives the answers from the printers.
Alternatively you can use a bridge network and manually configure your printer.

## Compose

A deployment using docker (or podman) compose might look like this:

```yaml
services:
  escpos2mqtt:
    image: ghcr.io/jhbruhn/escpos2mqtt:main
    restart: unless-stopped
    environment:
      MQTT_URL: "mqtt://rolf:12345678@192.168.1.5"
    network_mode: host
```

Note how the network mode is set to `host`!

## HomeAssistant Add-On

A HomeAssistant Add-On of this service is provided.
To install, add the repository available at `https://github.com/jhbruhn/escpos2mqtt-hassio` and install the escpos2mqtt add-on from there.
See the add-on there for further information on configuration.
The MQTT-broker is discovered automatically using the HA mechanisms.
That on the other hand means that the MQTT broker must also be managed by HomeAssistant.
