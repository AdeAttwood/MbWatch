<div align="center">

![mbwatch](assets/mbwatch-header.png)

</div>

mbwatch is a CLI tool designed to monitor mailboxes and automatically invoke
mbsync when changes occur in the remote mailbox. mbsync is a highly regarded
tool for synchronizing remote email accounts with local maildir, but it
requires manual execution to update changes. Typically, this is managed through
periodic polling using a tool like cron, which can result in delayed message
synchronization.

mbwatch addresses this issue by leveraging the IMAP IDLE feature to
continuously monitor the mailbox for changes. When a change is detected,
mbwatch automatically triggers mbsync to sync the new messages, ensuring your
local maildir is always up-to-date without manual intervention.

## Prerequisites

- The remote IMAP server must support [IDEL](https://en.wikipedia.org/wiki/IMAP_IDLE)
- The rust toolchain for building
- [mbsync](https://isync.sourceforge.io/mbsync.html) installed and configured

## Installation

Right now there are no prebuilt binary, you can install from git using cargo.

```bash
cargo install --git https://github.com/AdeAttwood/MbWatch
```

## Configuration

This tool will use the `mbsync` `Groups` config from your existing `.mbsyncrc`.
You will need to define a group called `mbwatch`, it will then watch all the
mailboxes defined in the `Channels` prop. This should be a comma separated list
of `channel:mailbox` like you define on the mbsync cli.

This is an example config, it assumes you have two channels called `work` and
`personal` each remote store has a mailbox called `INBOX` and the `work`
mailbox has one called `Archive`.

```conf
Group mbwatch
Channels work:INBOX, work:Archive, personal:INBOX
```

For more details on the `.mbsyncrc` you can view the upstream [mbsync docs](https://isync.sourceforge.io/mbsync.html#CONFIGURATION)



