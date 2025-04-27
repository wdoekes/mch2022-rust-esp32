hellomch
========

*TODO: Document this.*


-------------
Prerequisites
-------------

- ``cargo install espup``

- ``espup install``

  This creates a ``~/export-esp.sh`` file. You might want to move it
  into this project directory, or somewhere not directly in your
  homedir.

- Now, before building anything you meed to source ``export-esp.sh``:

  ``. /path/to/export-esp.sh``

- This project was initially generated with:

  ``esp-generate --chip esp32 hellomch``

  In the configuration *enable unstable-hal* and *enable wifi* were set.

  You don't need to do this, since you already have this project checked out.

Now you can run ``make`` to build:

.. code-block:: console

    $ make
    ...
    cargo build --release
       Compiling hellomch v0.1.0 (/home/walter/Arduino/projmch2022/mch2022-rust-esp32)
        Finished `release` profile [optimized + debuginfo] target(s) in 4.32s

Check the *Flashing* section below to write the firmware over the launcher.


--------
Flashing
--------

Flashing the ESP32 on the *MCH2022 Badge* should be a matter of typing ``make flash``:

.. code-block:: console

    $ make flash
    espflash flash --chip=esp32 --port=/dev/ttyACM0 -M --partition-table=partitions.csv \
      --flash-size=16mb --target-app-partition=ota_0 \
      ./target/xtensa-esp32-none-elf/debug/hellomch
    [2025-04-27T17:21:06Z INFO ] Serial port: '/dev/ttyACM0'
    [2025-04-27T17:21:06Z INFO ] Connecting...
    [2025-04-27T17:21:08Z INFO ] Using flash stub
    Chip type:         esp32 (revision v3.0)
    Crystal frequency: 40 MHz
    Flash size:        16MB
    Features:          WiFi, BT, Dual Core, 240MHz, Coding Scheme None
    MAC address:       40:f5:20:57:08:a8
    Partition table:   partitions.csv
    App/part. size:    125,120/1,638,400 bytes, 7.64%
    ...
    [2025-04-27T17:21:09Z INFO ] Flashing has completed!
    Commands:
        CTRL+R    Reset chip
        CTRL+C    Exit

But, you may sometimes get a ``espflash::timeout`` instead:

.. code-block:: console

    $ make flash
    ...
    [2025-04-27T17:19:25Z INFO ] Using flash stub
    Error: espflash::timeout

      × Error while connecting to device
      ╰─▶ Timeout while running command

    make: *** [Makefile:20: flash] Error 1

This has been observed with ``espflash 3.3.0`` (with ``rustc
1.85.0-nightly``). Generally, rerunning ``make flash`` a couple of times
is sufficient to make it work.


-------------------
Restoring the badge
-------------------

If you want to restore the *MCH2022 Badge* to its original glory, you can fetch:

- [launcher.elf](https://github.com/badgeteam/mch2022-firmware-esp32/releases/download/v2.0.5/launcher.elf)

Flash this using:

.. code-block:: console

   $ espflash flash --chip=esp32 --port=/dev/ttyACM0 -M --partition-table=partitions.csv \
     --flash-size=16mb --target-app-partition=ota_0 launcher.elf

*Note that it flashes about 1.6MiB of binary, not the entire 16MiB ELF file.*

----

If you were also writing to the *RPi 2040* you'll need to fetch that as well:

- [rp2040_firmware.bin](https://github.com/badgeteam/mch2022-firmware-esp32/raw/refs/tags/v2.0.9/resources/rp2040_firmware.bin)

- [rp2040.uf2](https://github.com/badgeteam/mch2022-autoflasher/raw/refs/heads/master/rp2040/rp2040.uf2)

Flash this using:

- Hold SELECT while powering on. The *badge* will start in *RPi*
  flashing mode. *You should see a red flashing kite.* Now you can copy
  ``rp2040.uf2`` to ``/media/YOURUSER/RPI-RP2/``. Maybe. 

- Or, you can hold MENU while powering on. The *badge* will rewrite the
  RP2040 co-processor firmware automatically.
