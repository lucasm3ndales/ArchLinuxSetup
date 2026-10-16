# Acer Aspire VX5-591G

This profile covers the Aspire VX5-591G family regardless of regional model suffixes. ArchSetup compares `Acer` with the DMI vendor and `VX5-591G` with the model name. The profile applies only when you pass `--profile acer-aspire-vx5-591g`.

```bash
archsetup doctor --profile acer-aspire-vx5-591g
archsetup pkg --profile acer-aspire-vx5-591g --dry-run
archsetup desktop --profile acer-aspire-vx5-591g --dry-run
```

`doctor` may suggest this profile, but it never selects it automatically. Review the preview before installing. `desktop` requires an installed Xfce desktop and an active Xfce X11 session.

## What the profile adds

| Category | Additional packages |
| --- | --- |
| `gaming` | `lib32-mesa`, `lib32-vulkan-intel`, `nvidia-580xx-dkms`, `nvidia-580xx-utils`, `lib32-nvidia-580xx-utils` |

`archsetup desktop --profile acer-aspire-vx5-591g` enables tap-to-click on the touchpad. Natural scrolling is a shared `desktop` setting and also works without a profile.

The packages and setting are declared in [profile.toml](profile.toml). `pkg gaming` selects headers for the installed kernels because the category includes an NVIDIA DKMS driver; `pkg base` alone does not install them. For `linux` and `linux-zen`, it selects `linux-headers` and `linux-zen-headers`. The `gaming` category requires `[multilib]`. The shared `desktop` category includes Blueman for Bluetooth management and Docklike for grouped window previews; the 32-bit Intel and NVIDIA packages support games using either GPU.

## Before using the profile

The DMI identifier covers the base model, but variants may have different components. **Use `pkg gaming` with this profile only if the machine has an Intel GPU and an NVIDIA GTX 10xx/Pascal GPU**, such as the GTX 1050. Check with `lspci -nn | grep -Ei 'VGA|3D'`. ArchSetup rejects conflicting official NVIDIA drivers and does not remove them automatically. The 580xx branch is distributed through the AUR; review the PKGBUILD and DKMS build before rebooting. The profile does not configure PRIME, boot, encryption, power management, or snapshots. To use the dedicated GPU under X11, consult the Arch PRIME documentation after installing the driver.

The profile does not reinstall or replace Btrfs, LUKS, Snapper, zram, Limine, NetworkManager, PipeWire, or Bluetooth, printing, power, and firewall services selected during system installation. Already installed packages are skipped with `--needed`. `desktop` configures Xfce and the touchpad and adjusts Ly when it is enabled.
