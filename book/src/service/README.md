# Service

The service does the communication part between hardware and software,
by sending the commands received from the D-Bus API to the ACPI.

## D-Bus API

The service can be controlled through the D-Bus API.
You can also view information like the temperature, fan speed, etc.

```xml
{{ #include ../../../interfaces/fancy.xml }}
```

### Properties changes

It's not possible (for now) to subscribe to the following properties:

- `FansSpeeds`
- `PollInterval`
- `FansNames`
- `Critical`
- `FansNames`
- `Temperatures`

This is because they are directly modified by the service
and the signal `org.freedesktop.DBus.Properties.PropertiesChanged`
have to be triggered manually, which would consume more CPU.

`FansNames` and `PollInterval` change only when `Config` is changed,
so subscribing to `Config`
and getting these properties when it changes should work.
The other properties can instead manually be polled
with an interval of `PollInterval`.
