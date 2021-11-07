# Debugging

To debug the service, you can set the `RUST_LOG` variable
and check the logs.

## systemd

With systemd, you can do that by running `systemctl edit fancy.service`
and edit the file like this:

```ini
### Editing /etc/systemd/system/fancy.service.d/override.conf
### Anything between here and the comment below will become the new contents of the file
[Service]
# Replace debug by the appropriate level
Environment="RUST_LOG=debug"

### Lines below this comment will be discarded
```

You can then check the logs by running `journalctl -xeu fancy.service`.
If you want to follow the logs, you can run `journalctl -feu fancy.service`.

## Log levels

The possible log levels are:

- debug
- info
- trace
