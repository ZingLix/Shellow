# 商店发布流水线配置说明

这个项目的发布流水线在 `.github/workflows/release-mobile.yml`。

流水线会做这些事：

- Android：编译 AAB，并上传到 Google Play closed testing。默认 track 是 `alpha`。
- iOS：编译归档并上传到 App Store Connect，处理完成后会出现在 TestFlight。
- 手动触发：GitHub -> Actions -> Release Mobile -> Run workflow。
- Tag 触发：推送 `v*` tag，例如 `v0.1.1`，会同时发布 Android 和 iOS。

## 1. GitHub 环境

建议在 GitHub 仓库里创建一个 Environment：

```text
Settings -> Environments -> New environment -> mobile-release
```

如果你希望真正上传商店前需要人工确认，可以给这个 environment 配 required reviewers。
不配置保护规则也可以跑。

所有密钥都放这里：

```text
Settings -> Secrets and variables -> Actions -> New repository secret
```

如果你用了 `mobile-release` environment，也可以放到 environment secrets。

## 2. Android / Google Play 配置

Android 包名是：

```text
xyz.zinglix.shellow
```

### 2.1 先在 Play Console 做的事

1. 在 Google Play Console 创建应用。
2. 包名使用 `xyz.zinglix.shellow`。
3. 开启 Play App Signing。
4. 创建或确认 closed testing track。
5. 默认 workflow 使用 `alpha` 作为 closed testing track。

如果你的 closed testing track 是自定义名字，不叫 `alpha`，需要改
`.github/workflows/release-mobile.yml` 里的 `android_track` 默认值和 options。

### 2.2 生成 Android upload key

如果还没有 upload keystore，可以在本机生成：

```sh
keytool -genkeypair \
  -v \
  -keystore shellow-upload.jks \
  -alias shellow-upload \
  -keyalg RSA \
  -keysize 4096 \
  -validity 10000
```

把 keystore 转成 base64，放进 GitHub Secret：

macOS：

```sh
base64 -i shellow-upload.jks | pbcopy
```

Linux：

```sh
base64 -w 0 shellow-upload.jks
```

### 2.3 Android 需要的 GitHub Secrets

```text
ANDROID_KEYSTORE_BASE64
ANDROID_KEYSTORE_PASSWORD
ANDROID_KEY_ALIAS
ANDROID_KEY_PASSWORD
GOOGLE_PLAY_SERVICE_ACCOUNT_JSON
```

含义：

```text
ANDROID_KEYSTORE_BASE64        shellow-upload.jks 的 base64 内容
ANDROID_KEYSTORE_PASSWORD      keystore 密码
ANDROID_KEY_ALIAS              key alias，例如 shellow-upload
ANDROID_KEY_PASSWORD           key 密码
GOOGLE_PLAY_SERVICE_ACCOUNT_JSON  Google service account JSON 的完整内容
```

### 2.4 配置 Google Play API 上传权限

1. 打开 Google Cloud Console。
2. 找到和 Play Console 关联的项目。
3. 启用 Google Play Android Developer API。
4. 创建 service account。
5. 给 service account 创建 JSON key。
6. 把 JSON 文件完整内容放到 `GOOGLE_PLAY_SERVICE_ACCOUNT_JSON`。
7. 回到 Google Play Console，邀请这个 service account 的邮箱。
8. 给它 Shellow 应用的 release/upload 权限。

### 2.5 Android versionCode

流水线会用 GitHub 的 `GITHUB_RUN_NUMBER` 作为 `versionCode`，所以每次跑
`Release Mobile` 都会递增。

如果你以前已经手动上传过很大的 `versionCode`，例如 1000，而 workflow 第一次跑是
1，那么需要给 workflow 加偏移量，或者先把 workflow 跑到足够大的 run number。

`versionName` 的规则：

- 手动触发时填写 `version_name`，就用这个值。
- 推送 tag `v0.1.1` 时，使用 `0.1.1`。
- 都没填时，使用 Android 工程里的默认值 `0.1.0`。

## 3. iOS / TestFlight 配置

iOS bundle identifier 是：

```text
xyz.zinglix.shellow
```

流水线会上传到 App Store Connect。Apple 处理完成后，这个 build 会出现在 TestFlight。
它不会自动提交正式 App Store 审核。

### 3.1 先在 Apple 后台做的事

1. Apple Developer 创建 App ID / Bundle ID：`xyz.zinglix.shellow`。
2. App Store Connect 创建应用记录。
3. 创建 Apple Distribution certificate。
4. 创建 App Store provisioning profile，bundle id 选择 `xyz.zinglix.shellow`。
5. 创建 App Store Connect API key。

### 3.2 导出 Apple Distribution 证书

从 Keychain Access 导出 Apple Distribution 证书为 `.p12` 文件，例如：

```text
distribution.p12
```

转成 base64：

macOS：

```sh
base64 -i distribution.p12 | pbcopy
```

Linux：

```sh
base64 -w 0 distribution.p12
```

### 3.3 编码 provisioning profile

下载 App Store provisioning profile，例如：

```text
Shellow_App_Store.mobileprovision
```

转成 base64：

macOS：

```sh
base64 -i Shellow_App_Store.mobileprovision | pbcopy
```

Linux：

```sh
base64 -w 0 Shellow_App_Store.mobileprovision
```

### 3.4 编码 App Store Connect API key

在 App Store Connect 创建 API key 后，会下载一个 `.p8` 文件，例如：

```text
AuthKey_XXXXXXXXXX.p8
```

转成 base64：

macOS：

```sh
base64 -i AuthKey_XXXXXXXXXX.p8 | pbcopy
```

Linux：

```sh
base64 -w 0 AuthKey_XXXXXXXXXX.p8
```

### 3.5 iOS 需要的 GitHub Secrets

```text
IOS_TEAM_ID
IOS_DISTRIBUTION_CERTIFICATE_BASE64
IOS_DISTRIBUTION_CERTIFICATE_PASSWORD
IOS_PROVISIONING_PROFILE_BASE64
APP_STORE_CONNECT_API_KEY_ID
APP_STORE_CONNECT_ISSUER_ID
APP_STORE_CONNECT_API_KEY_P8_BASE64
```

含义：

```text
IOS_TEAM_ID                         Apple Developer Team ID
IOS_DISTRIBUTION_CERTIFICATE_BASE64 distribution.p12 的 base64 内容
IOS_DISTRIBUTION_CERTIFICATE_PASSWORD  导出 p12 时设置的密码
IOS_PROVISIONING_PROFILE_BASE64     App Store mobileprovision 的 base64 内容
APP_STORE_CONNECT_API_KEY_ID        App Store Connect API key 的 Key ID
APP_STORE_CONNECT_ISSUER_ID         App Store Connect API 的 Issuer ID
APP_STORE_CONNECT_API_KEY_P8_BASE64 AuthKey_XXXXXXXXXX.p8 的 base64 内容
```

## 4. 怎么发布

### 手动发布

1. 打开 GitHub 仓库。
2. 进入 Actions。
3. 选择 `Release Mobile`。
4. 点 `Run workflow`。
5. `platform` 选择：
   - `all`：Android 和 iOS 都发
   - `android`：只发 Google Play
   - `ios`：只发 TestFlight
6. `android_track` 默认是 `alpha`。
7. `version_name` 可以填，例如 `0.1.1`。

### Tag 发布

```sh
git tag v0.1.1
git push origin v0.1.1
```

推送 tag 会同时发布 Android 和 iOS，并把 `0.1.1` 作为显示版本号。

## 5. 发布后去哪里看

Android：

```text
Google Play Console -> Testing -> Closed testing
```

iOS：

```text
App Store Connect -> TestFlight
```

第一次发布时两个平台都可能要求补齐隐私、年龄分级、合规、测试人员等信息。
