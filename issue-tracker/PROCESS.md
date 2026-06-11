# Issue Tracker - Process Documentation

## Build Process

### Design Doc
Since the syllabus contained a design outline, I took that directly to an agent discussion.  That went well and pointed out a few gaps in the doc, such as not defining what format the <id> value would use, or how exactly to store the `created_at` value.  With that handled, I got to work.  

### Initial Build
I built out this project perhaps a bit too quickly.  The directions to validate each layer weren't completely clear and I didn't know how to build and install, so I wasn't able to validate.  I got the pieces built, and injected the directions to validate input along the way.  It went fine, with one error that I noted, but considered possibly useful.  That was that the list displayed all _non-closed_ items rather than **just open** issues.  As a result when I started testing, I noticed the issue.  I have made a distinction between `done` and `closed` items in my schema. So the list will show `done` items in the default list without sorting.  

Once I was finished creating the tracker, I figured out how to install it and ran it through some testing.  I found a few sticking points and started putting issues into the tracker for later recall.  This is when I noticed a few more issues, and added them to the tracker too.  That gives me some usability and feature issues to tackle, and now I'm ready to run my first review.

## Issues and Review
At this point, I'll be storing the issues in the tracker for the duration of the build and documenting thought process here, primarily.  I'll fill in my experience, and keep the issues in that location.  

### Review 1

#### Review 1 Notes
There are a number of issues with the command interface. Either missing features like not being able to set a description at all, or edit a title.  Other issues included a lack of `label` normalizing, and `priority` accepting arbitrary strings, while needing to sort by that value.  As a result, I'm wondering if some of those could be simplified with a config file?  I think there's also something to be said for the ergonomics of giving shorter commands too, especially for status moves. The issue would be that those commands would change if the status options changed, so maybe that's best left to shell aliases?  I think the first step would be to hardcode the statuses, and step through them in order to handle the process of moving through the states in the project?

There's a lot of ergonomics and workflow questions here.  I'll probably need another planning session to sort this out.  I'm also considering a pre-loaded list of labels, set in the config, defaulting to semver.  This _could_ have an interactive entry mode for a TUI, and also live filtering and sorting, but I think that's probably an overbuild for a basic assignment that won't be actively used.  

#### Review 1 Fixes
This session was fascinating, and different.  Instead of pointing to issues in the process doc, the agent stepped through using the tracker itself and that helped a lot, especially when it added issues with the interface as it found them.  I can understand now how you can plan a project and let things run without needing to do a ton of direction on the specific steps as things run.

The tracker had some real structural and interface issues, and the updates really helped a ton.  I started getting more and more ideas of things to do with it.

### Review 2

##### Review 2 Notes
This review is mostly finding small issues, and a good amount of the feedback is about my docs lol.  I'm holding myself back from adding features and trying to do a final round of polish before submitting for review and testing out crosslink.  I'm excited to get to the point where I feel comfy enough in the basics to use real tools, but I'm getting the urge to build all my tooling from scratch to fit my style.  I'm resisting that because it's just going to slow me down and I'm still trying to learn from others' mental models.  I'll build my own once I start feeling limited.

In the tracker, this review's issues start at #31.