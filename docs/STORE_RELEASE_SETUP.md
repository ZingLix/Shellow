# Store Release Setup

This project ships native Android and iOS apps from GitHub Actions through
`.github/workflows/release-mobile.yml`.

Chinese setup notes are available in `docs/STORE_RELEASE_SETUP.zh-CN.md`.

The workflow can be started manually from GitHub Actions, or by pushing a tag
named `v*`, such as `v0.1.1`. Manual runs let you choose `all`, `android`, or
`ios`, plus the Google Play track. Android defaults to the `alpha` track, which
is the usual Google Play API track name for closed testing.

## GitHub environment

The release jobs use the `mobile-release` GitHub Environment. Create it under
repository Settings -> Environments if you want required reviewers before a
store upload. If you do not add protection rules, the workflow still runs.

Add all values below under repository Settings -> Secrets and variables ->
Actions.

## Android / Google Play

The Android package name is:

```text
xyz.zinglix.shellow
```

Before CI can upload, create the app in Google Play Console and upload the first
signed build manually if Play Console requires it for the package.

The workflow defaults to `alpha` for closed testing. If your Play Console closed
testing track uses a custom track name, replace the `alpha` default/options in
`.github/workflows/release-mobile.yml` with that exact track name.

Required GitHub secrets:

```text
ANDROID_KEYSTORE_BASE64
ANDROID_KEYSTORE_PASSWORD
ANDROID_KEY_ALIAS
ANDROID_KEY_PASSWORD
GOOGLE_PLAY_SERVICE_ACCOUNT_JSON
```

Create an upload keystore once if you do not already have one. This is the
Google Play upload key, not the final app signing key that Google Play uses to
sign APKs delivered to users. Keep the original keystore outside the repository,
back it up somewhere durable, and reuse it for every future Android release.
Do not regenerate it per build or per release.

```sh
keytool -genkeypair \
  -v \
  -keystore shellow-upload.jks \
  -alias shellow-upload \
  -keyalg RSA \
  -keysize 4096 \
  -validity 10000
```

Encode it for GitHub Secrets:

```sh
base64 -i shellow-upload.jks | pbcopy
```

Use that output as `ANDROID_KEYSTORE_BASE64`. `ANDROID_KEY_ALIAS` should match
the alias above unless you used a different one.

If Google Play asks you to register or reset the upload key, export the public
upload certificate from the same keystore:

```sh
keytool -export -rfc \
  -keystore shellow-upload.jks \
  -alias shellow-upload \
  -file shellow-upload-certificate.pem
```

The `.pem` file is public certificate material and can be uploaded to Play
Console when prompted. The `.jks` file and passwords are private signing
material and must stay secret.

For Play upload access:

1. Enable the Google Play Android Developer API for the Google Cloud project.
2. Create a service account and JSON key.
3. Invite the service account email in Google Play Console.
4. Grant it release permissions for the Shellow app.
5. Paste the full JSON file contents into `GOOGLE_PLAY_SERVICE_ACCOUNT_JSON`.

CI sets `versionCode` from `GITHUB_RUN_NUMBER`. If you pass `version_name`
manually, or push a tag like `v0.1.1`, CI uses that for `versionName`.

## iOS / App Store Connect

The iOS bundle identifier is:

```text
xyz.zinglix.shellow
```

Before CI can upload, create the app record in App Store Connect and make sure
the bundle identifier exists in Apple Developer.

Required GitHub secrets:

```text
IOS_TEAM_ID
IOS_DISTRIBUTION_CERTIFICATE_BASE64
IOS_DISTRIBUTION_CERTIFICATE_PASSWORD
IOS_PROVISIONING_PROFILE_BASE64
APP_STORE_CONNECT_API_KEY_ID
APP_STORE_CONNECT_ISSUER_ID
APP_STORE_CONNECT_API_KEY_P8_BASE64
```

Create or export an Apple Distribution certificate as a `.p12`, then encode it:

```sh
base64 -i distribution.p12 | pbcopy
```

Use that output as `IOS_DISTRIBUTION_CERTIFICATE_BASE64`, and use the `.p12`
export password as `IOS_DISTRIBUTION_CERTIFICATE_PASSWORD`.

Create an App Store provisioning profile for `xyz.zinglix.shellow`, download the
`.mobileprovision`, and encode it:

```sh
base64 -i Shellow_App_Store.mobileprovision | pbcopy
```

Use that output as `IOS_PROVISIONING_PROFILE_BASE64`.

Create an App Store Connect API key with permission to upload builds. Encode the
downloaded `.p8` key:

```sh
base64 -i AuthKey_XXXXXXXXXX.p8 | pbcopy
```

Use that output as `APP_STORE_CONNECT_API_KEY_P8_BASE64`. The key ID and issuer
ID become `APP_STORE_CONNECT_API_KEY_ID` and `APP_STORE_CONNECT_ISSUER_ID`.

CI uploads the archive to App Store Connect for TestFlight processing. After
Apple finishes processing, the build appears in TestFlight/App Store Connect.
Submitting the build for App Store review still requires store metadata, export
compliance, privacy answers, and release choices in App Store Connect unless you
automate those separately.

## Running a release

Manual:

1. Open GitHub -> Actions -> Release Mobile.
2. Choose `Run workflow`.
3. Pick platform and Google Play track.
4. Optionally set `version_name`, for example `0.1.1`.

Tag based:

```sh
git tag v0.1.1
git push origin v0.1.1
```

The tag path releases both Android and iOS using the tag without the leading
`v` as the marketing version.
