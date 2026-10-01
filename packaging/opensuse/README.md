# openSUSE Packaging for KKFetch

This directory contains Open Build Service (OBS) packaging files for openSUSE Tumbleweed and openSUSE Leap.

## Files

- [`kkfetch.spec`](kkfetch.spec): RPM specification tailored for openSUSE build targets.
- [`_service`](_service): OBS Source Service configuration to fetch source tarballs directly from GitHub releases.

## OBS Setup

1. Register or log in at [build.opensuse.org](https://build.opensuse.org).
2. Your home project is `home:<username>`.
3. Create a package named `kkfetch` under your home project.
4. Add build targets:
   - `openSUSE_Tumbleweed`
   - `openSUSE_Leap_15.6`
5. Upload `kkfetch.spec` and `_service`.
6. Once the build completes, the repository is automatically published at:
   `https://download.opensuse.org/repositories/home:/<username>/openSUSE_Tumbleweed/`

## User Installation

```bash
sudo zypper ar -f https://download.opensuse.org/repositories/home:/<username>/openSUSE_Tumbleweed/home:<username>.repo
sudo zypper refresh
sudo zypper install kkfetch
```
