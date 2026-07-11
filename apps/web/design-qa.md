# Shellow landing page design QA

- Source visual truth: approved Sillage landing-page reference
- Implementation URL: `http://127.0.0.1:4173/`
- Intended viewports: desktop 1440 × 1000; mobile 390 × 844
- State: default landing page with Codex selected in the coding-agents section
- Simulator evidence:
  - `public/screens/ios-hosts.jpg`
  - `public/screens/ios-terminal.jpg`

**Findings**

- No blocking visual or interaction defects found in the reviewed desktop and mobile viewports.

**Required fidelity surfaces**

- Fonts and typography: verified at 1440 × 1000 and 390 × 844 with the native system stack and bundled JetBrains Mono.
- Spacing and layout rhythm: verified across the desktop and mobile breakpoints with no horizontal overflow.
- Colors and visual tokens: rendered dark/light sections match Shellow's native palette with readable contrast.
- Image quality and asset fidelity: both product images are isolated iOS Simulator captures using sanitized fixture data; loading and final browser crops were verified.
- Copy and content: verified for desktop and mobile wrapping across the full landing-page narrative.

**Full-view comparison evidence**

- Desktop and mobile full-page layouts were reviewed in the in-app browser at the intended viewports.

**Focused region comparison evidence**

- The two source product screenshots contain only fixture hosts and example domains. Hero, performance, coding-agent, and FAQ regions were inspected in-browser.

**Primary interactions tested**

- HTTP response for the page: passed.
- HTTP responses for both screenshots and the app icon: passed.
- Browser navigation and FAQ disclosure controls: passed.
- Broken image, horizontal overflow, and browser console checks: passed.

**Implementation Checklist**

- Desktop and mobile implementations reviewed.
- Image crops and above-the-fold hierarchy checked.
- Navigation and FAQ disclosure controls tested.
- No P0/P1/P2 visual findings remain.

**Follow-up Polish**

- Evaluate whether the hero device composition should become slightly denser after the first desktop capture.
- Confirm whether the terminal screenshot should retain the on-screen keyboard on wide desktop layouts.

**Comparison history**

- Initial pass: implementation build and asset delivery succeeded, but browser evidence was unavailable.
- Review pass: desktop and mobile browser evidence completed; layout, assets, responsive behavior, FAQ interaction, and console checks passed.

final result: passed
