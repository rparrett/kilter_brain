# Kilter Helper

## TODO

- Automatic updates from API
- Add route authoring
- Let users change the angle of the board
- web: Figure out how to handle pastes when the canvas is focused
- Make frame parser more permissive

## To get a recent APK

- Download in play store on real phone
- Connect real phone over USB
- `adb shell pm list packages -f -3`
- Remove `package:` prefix and `=com.auroraclimbing.kilterboard` suffix.
- `adb -d pull /data/app/~~q5QrKuSQ1h9lR6-cOVPeoQ==/com.auroraclimbing.kilterboard-nGUgTfgrHXqe-bWe5zRCiQ==/base.apk`
- Move to `Downloads/kilter-apk/(version)`
- `unzip base.apk`
