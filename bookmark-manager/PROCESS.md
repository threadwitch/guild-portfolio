# Bookmark Manager - Process Documentation

## Build Process

### Initial Prompt

> This is the design document for a personal bookmark manager project I'm working on. Let's start working on the core. I want to be able to add a bookmark with a URL and title, and have that saved so it persists when I load the page. Like the design doc says, use local browser storage to save the data. Remember that we're only using plain HTML, CSS, and JavaScript in a single index.html file. Make the design neat and minimal.

This *mostly* worked on the first try. There were some issues with URL input validation that took a few rounds to fix. Specifically, it was allowing URLs with `https:///` to validate with the third slash. That was solved after two or three corrective passes. This prompt also implementd a lot of the design doc in a single shot. Tags and notes were complete and correctly saving from the first prompt, as well as search and tag filtering.

### Second feature prompt

> Alright, now that we have that working, let's add bookmark editing. Clicking on the edit button should let me edit the title, URL, notes, and tags inline. Make sure the changes persist in local storage when saving the edits.

The second feature prompt was to add an edit button. Delete was in place from the start. This worked immediately, and stood up to input validation tests, canceling properly and discarding edits, as well as persisting changes. I tested a variety of things, including fully clearing fields, and removing the title, and the code behaved as expected for all of it.

### UI Polish prompt

> The functionality is all working as intended. Now lets work on the interface. I want better typography using a monospace font. Let's adjust the color palette too. I'm thinking about a dark there in the vein of TokyoNight. I want to make sure the animations are smooth for filtering, searching, and editing. Also, extract the domain name from the URL and use that as a label.

With the functional parts accomplished with prompts than expected, I moved to the look and feel. I picked a known color palette that I like, and that was incorporated well. Overall, I'm pretty happy with it. I think the animations are a little bit abrupt still, so that's an area that could be improved, but I'm happy with the basics.

## What I've learned

Well, this went more smoothly and with less handholding than I expected. I was also pleasantly surprised with how easy the edits were. I think there's probably more I could learn and expand on in regards to redirection and correction, this build went a bit too smoothly to practice that.

Having experienced this, I feel more practically positive about future work with AI.

## Known Issues

The animations are pretty abrupt, to the point that I think they *may* not be rendering properly, but that'll take a bit more testing, and adversarial review.

## Review 1
The review resulted in 23 issues being raised.

### 1. Stored XSS via `innerHTML` in `makeTagButton`
This is an issue where the tag field is not sanitizing the input, but instead allow HTML injection through tag field input, and once loaded in, it's persistent. The actual risk is low, but only because it's tied to local access and there's an assumption that I'm the only user.

**Validity:** Valid
**Priority:** High
**Reasoning:** This represents a security flaw and should be rectified.

### 2. Silent Data Loss on `localStorage` Quota Exceeded
There's no error catching or warning when the localStorage data limit is reached or exceeded, resulting in silent failure to save a bookmarks and edits.

**Validity:** Valid
**Priority:** High
**Reasoning:** Silent save failures with no explanation are a major failure of function, and there's no indication of how likely it might be to occur.

### 3. Crash on Malformed `localStorage` Data
If the data in stored in localStorage is malformed JSON somehow, through a manual edit or format change, the app throws an error and crashes. The analysis says that it should crash if `b.notes` is a null value, but that seems to suggest that bookmarks without notes would cause a full crash, which isn't the case.

**Validity:** Partially valid, I think
**Priority:** High-medium
**Reasoning:** This seems mostly valid, though I think one of the examples is off. It also seems like it should be a straightforward fix, so it can be done early.

### 4. Ghost filter state after deleting the last bookmark in a tag
If you have a tag filtered, and delete the last tag, the filter stays applied, showing no bookmarks. The "save you first link text" is then shown, with no indicator of what's happened, or how to fix it. Selecting all tags will do the correct thing, but the user would have to guess that. In addition, the tag filter is gone from the sidebar, so there's no indicator at all that a filter could still be in play.

**Validity:** Valid
**Priority:** Medium-high
**Reasoning:** This is not *too* likely at the moment, but surfaces some unwanted behavior that's hard to trace for the user.

### 5. Wrong empty state message
There's only one empty state message, and it assumes that no bookmarks have been added. This then gives incorrect information when a search or filter returns no results.

**Validity:** Valid
**Priority:** High
**Reasoning:** This is an issue where the interface is basically just lying to the user in several common scenarios. It's also linked to issue 4.

### 6. No delete confirmation
The delete button doesn't confirm deletion and is close to the edit button, making mistakes likely.

**Validity:** Valid
**Priority:** Medium
**Reasoning:** This is a valid issue and can lead to inadvertent data loss. It should be corrected.

### 7. Enter key on cancel button triggers both cancel and save
If the cancel button is focused, and you press enter, it fires a save event and then a cancel event. That's not ideal because the behavior is potential opposite of what's wanted, and also potentially inconsistent. This is tied to bad modal focus.

**Validity:** Valid
**Priority:** Medium
**Reasoning:** This is an issue and unwanted behavior. It is also tied to an issue underlying a number of pieces of feedback, so addressing that issue should help with a number of listed issues.

### 8. No focus trap in the modal
Speaking of modal focus, this relates to the keyboard not being locked in to the modal, so this is an issue for screen readers, and also just keyboard navigation within the app.

**Validity:** Valid
**Priority:** Medium
**Reasoning:** Modal focus is an issue that connects to a few issues, and I would perfer good accessibility, and good UX in general, be incorporate into the project.

### 9. Inline edit labels are not associated with their inputs
There's structural issues with the labeling of the edit fields. This causes issues for screen readers because they don't give context to the edit fields.

**Validity:** Valid
**Priority:** Low
**Reasoning:** This isn't an immediate functional issue because I don't use a screen reader. We will deal with it before moving on, however, it's not functionally detrimental in the immediate.

### 10. Edit action buttons have no accessible names
This is related to 9, in the sense that it's an accessibility issue with the edit modal. The action buttons lack context, as in issue 9.

**Validity:** Valid
**Priority:** Low
**Reasoning:** Again, not a current functional break, and it's tied in with issue 9. They'll be dealt with almost at the same time.

### 11. Background content not hidden from assistive technology when modal is open
The modals don't set the correct properties in ARIA for a lot of screen readers. The background is still able to be paged through and that's suboptimal.

**Validity:** Valid
**Priority:** Low
**Reasoning:** Again, not a breaking change, it's a pure accessibility fix that will be done, but isn't first in line. This is possibly a pure restatement of issue 8.

### 12. No prefers-reduced-motion support
The CSS doesn't respect the accessibility flag.

**Validity:** Valid
**Priority:** Low
**Reasoning:** This is something I'd like to have for maximal accessibility, but it's lower priority, given that it's not effecting my use.

### 13. isValidUrl Rejects Legitimate URLs
The URL validation logic rejects URLs without a `.`, and this is a direct result of my testing. The examples they give are things that I don't think I want in the bookmarks anyway, but I'm going to ask for more information.

After further review, the work I did to make the validation reject URLs without dots was unnecessary, and it's perfectly fine to allow those to parse.

**Validity:** Valid
**Priority:** Medium
**Reasoning:** This does legitimately break the basic functionality of the bookmark manager in edge cases, and because it's edge cases, it's not a high priority.

### 14. activeTag is not reset when opening the add modal
If there's an activeTag when a bookmark is added, but the new bookmark doesn't have that tag, the bookmark is added and immediately filtered out of the view. Obviously, this could look like a failure to save, and that's confusing behavior. The add modal could reset the tag filter, or the latest added bookmark could be displayed regardless of the tag filter. I think I would prefer the accuracy of the tag filter being removed. Alternatively, the add modal could be pre-populated with the current tag, and if it's removed, and not present on saving, it could clear the filter.

**Validity:** Valid
**Priority:** Medium
**Reasoning:** This is a silent success in a way that's unclear to the user that that's the case.

### 15. Timestamp IDs can collide
The timestamp is the only unique ID, and it's at millisecond resolution. I don't think that's likely to cause collisions without programmatic input, which isn't currently planned. I would like to add *something* else, just in case, but it's not critical.

**Validity:** Valid
**Priority:** Low
**Reasoning:** This is unlikely to occur unless more features are added, but I think adding another input to the  IDs might make sense.

### 16. parseTags splits only on commas
This is assuming the comma delimiters are unintended, and space should also be a delimiter, based on user familiarity with other bookmark managers. Commas are a common delimiter, and, I think, more intuitive to more people.

**Validity:** Not valid
**Priority:** N/A
**Reasoning:** Comma delimiters was the intended result, not space delimiters.

### 17. Hardcoded color not using CSS custom property
`border: 1px solid #1e3a5f;` is a bit of code where the color is hardcoded instead of using `--variable`, and this could lead to it being missed if the color scheme is changed later.

**Validity:** Valid
**Priority:** Low
**Reasoning:** Consistency is good.

### 18. Animation is applied to an empty container
index.html:652-654:

```html
function fillCard(card, b) {
  card.innerHTML = '';
  animateSwap(card);
```

`card.innerHTML = ''` clears the card first. Then `animateSwap` removes and re-adds `.card--swap` (forcing a reflow on an empty element). Then content is appended. The animation fades in an already-empty card, causing a flash of empty space before content appears. This is the "abrupt animation" that I noted as a known issue — but the cause is here: the animation should run AFTER content is inserted, not before.

**Validity:** Valid
**Priority:** Low
**Reasoning:** This is an issue that I notice, but is largely cosmetic, so it's in the lower priority group.

### 19. Search input trim diverges from display
The search box shows white space around query terms while not including whitespace in the query itself.

**Validity:** Valid
**Priority:** Low
**Reasoning:** I don't typically use whitespace in searches with intent, but it happens sometimes, and aggressive trimming could cause some issues.

### 20. `createdAt` is stored but never used
Creation timing is stored for every bookmark, but never shown or used for anything, at least so far.

**Validity:** Valid
**Priority:** Low
**Reasoning:** Not pressing functionality, but having it displayed, and available to search or use as a formal filter would be good. We'll start with just displaying it for now.

### 21. `<aside>` has no accessible label
index.html:482-485:

```html
<aside>
  <h2>Tags</h2>
```
The `<aside>` element announces as "complementary" to screen readers. With multiple landmarks, users need labels to distinguish them. This should have `aria-label="Tag filter"` or `aria-labelledby` pointing to the `<h2>`.

**Validity:** Valid
**Priority:** Low
**Reasoning:** Another presently invisible accessibility issue. I want to take care of it, since it's low effort to fix, just not the immediate priority.

### 22. No keyboard submission in the inline edit form
The add modal takes keyboard input just fine and is keyboard navigable. The inline edit form doesn't. That's a feature I want, and is inconsistent.

**Validity:** Valid
**Priority:** Medium
**Reasoning:** Key missing functionality and it's inconsistent.

### 23. PROCESS.md misattributes the "Known Issues" scope
The "Known Issues" section only includes the animation issue, and none of the other issues that the reviewer identified.

**Validity:** Not Valid
**Priority:** N/A
**Reasoning:** This is flagging the fact that I wasn't aware of errors as an error in and of itself, and this is rectified by the review.

---

## Review 1 — Remediation

All 22 valid issues from Review 1 were resolved in a single session. Issues 16 and 23 were marked not valid and left as-is.

### What was fixed

#### Security
- **1.** Replaced `innerHTML` in `makeTagButton` with DOM methods, closing the stored XSS vector via tag input.

#### Data integrity
- **2.** Wrapped `localStorage.setItem` in a try/catch; quota errors now surface a toast notification rather than silently failing.
- **3.** Added `normalizeBookmark` called on every `load()`, coercing missing or malformed fields to safe defaults and rejecting non-array stored values entirely.
- **15.** Replaced `Date.now()` IDs with `crypto.randomUUID()` to eliminate the millisecond collision window.

##### Functional bugs
- **4.** Reset `activeTag` to `'All'` in `renderSidebar` when the active tag no longer exists in the bookmark set.
- **5.** Empty state now shows context-appropriate messages: no bookmarks, no search results, or no results for the active tag filter.
- **13.** Removed the `hostname.includes('.')` check from `isValidUrl`; `localhost` and bare hostnames are now accepted.
- **14.** Reset `activeTag` to `'All'` on successful add so a newly saved bookmark is always visible.
- **18.** Moved `animateSwap` to after content is inserted in both `fillCard` and `enterEditMode`, fixing the flash-of-empty-space that was the original known animation issue.
- **19.** Stored raw search input in `searchQuery` and trimmed only at point of use, keeping display and match behaviour in sync.
- **20.** `createdAt` is now displayed on each card in local-timezone ISO-8601 format (`YYYY-MM-DD`).

#### UX
- **6.** Delete button now shows an inline confirmation step (Delete? / Cancel / Confirm) before committing.
- **7.** Enter-to-save in the add modal is now restricted to the URL, Title, and Tags inputs; focused buttons no longer race with the save handler.
- **14.** (See above.)
- **22.** Inline edit form now supports Enter to save (on URL, Title, Tags) and Escape to cancel on all fields, matching the add modal's keyboard behaviour.

#### Accessibility
- **8.** Added a Tab/Shift+Tab focus trap to the add modal.
- **9.** Inline edit labels now have `for`/`id` associations via a `fieldName` parameter in `makeEditField`.
- **10.** All action buttons (card Edit/Delete, inline edit Cancel/Save, delete confirmation) now carry descriptive `aria-label` values including the bookmark title.
- **11.** `<header>` and `.body` are set `inert` while the modal is open, hiding background content from assistive technology. Focus returns to the Add button on close.
- **12.** Added `@media (prefers-reduced-motion: reduce)` block collapsing all animation and transition durations.
- **21.** Added `aria-labelledby` to `<aside>` pointing to the Tags `<h2>`.

#### Code quality
- **17.** Extracted the hardcoded tag border colour `#1e3a5f` into a `--tag-border` CSS custom property alongside `--tag-bg` and `--tag-color`.

