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