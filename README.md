# Kilter Helper

## TODO

- Automatic updates from API
- Add route authoring
  - [X] Edit placements
  - [ ] Edit name, setter name
- Display difficulty, quality, ascents, etc in UI
- web: Figure out how to handle pastes when the canvas is focused
- web: Add the neat JS progress bar from the Bevy example showcase
- Make frame parser more permissive
- Let users search the routes database for other climbs with the selected holds
- Add zooming and panning where zooming all the way out also re-centers you
- Let users change the angle of the board
- Add a measuring tape
- Add a "max armspan required" stat?
  Is this even possible? Do delaunay, then djikstras, then find the longest edge of the shortest path?

## To get a recent APK

- Download in play store on real phone
- Connect real phone over USB
- `adb shell pm list packages -f -3`
- Remove `package:` prefix and `=com.auroraclimbing.kilterboard` suffix.
- `adb -d pull /data/app/~~q5QrKuSQ1h9lR6-cOVPeoQ==/com.auroraclimbing.kilterboard-nGUgTfgrHXqe-bWe5zRCiQ==/base.apk`
- Move to `Downloads/kilter-apk/(version)`
- `unzip base.apk`
