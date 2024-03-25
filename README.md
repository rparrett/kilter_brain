# Kilter Helper

## TODO

- Automatic updates from API
- Add route authoring
- Let users change the angle of the board
- web: Figure out how to handle pastes when the canvas is focused
- Make frame parser more permissive
- Let users search the routes database for other climbs with the selected holds
- Add button to clear the kilter data
- Add left/right arrow navigation of climbs
- Add zooming and panning where zooming all the way out also re-centers you

## To get a recent APK

- Download in play store on real phone
- Connect real phone over USB
- `adb shell pm list packages -f -3`
- Remove `package:` prefix and `=com.auroraclimbing.kilterboard` suffix.
- `adb -d pull /data/app/~~q5QrKuSQ1h9lR6-cOVPeoQ==/com.auroraclimbing.kilterboard-nGUgTfgrHXqe-bWe5zRCiQ==/base.apk`
- Move to `Downloads/kilter-apk/(version)`
- `unzip base.apk`
