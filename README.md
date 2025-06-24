# Kilter Brain

An alternative Kilter Board climb editor / viewer.

AI route-setting via [`kilter_brain_gen`](https://github.com/rparrett/kilter_brain_gen).

## TODO

- native: Automatic updates from kilter API
- Add route authoring
  - [X] Edit placements
  - [ ] Edit name, setter name, description, etc.
- Display difficulty, quality, ascents, etc in UI
- web: Figure out how to handle pastes when the canvas is focused
- Add asset / pipeline loading screen
- Add generic UI toast to show frame parsing errors, network errors, paste results, etc
- Make frame parser more permissive to whitespace?
- Let users search the routes database for other climbs with the selected holds
- Board angle setting should affect the appearance of the board
- Add a measuring tape
- Add a "min armspan required" stat?
  Is this even possible? Do delaunay, then djikstras, then find the longest edge of the shortest path?
- Make the "dude for scale" a ragdoll that can be dragged onto the board

## To get a recent APK

- Download in play store on real phone
- Connect real phone over USB
- `adb shell pm list packages -f -3`
- Remove `package:` prefix and `=com.auroraclimbing.kilterboard` suffix.
- `adb -d pull /data/app/~~q5QrKuSQ1h9lR6-cOVPeoQ==/com.auroraclimbing.kilterboard-nGUgTfgrHXqe-bWe5zRCiQ==/base.apk`
- Move to `Downloads/kilter-apk/(version)`
- `unzip base.apk`
