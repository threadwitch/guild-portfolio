# Issue Tracker - Process Documentation

## Build Process

### Design Doc
Since the syllabus contained a design outline, I took that directly to an agent discussion.  That went well and pointed out a few gaps in the doc, such as not defining what format the <id> value would use, or how exactly to store the `created_at` value.  With that handled, I got to work.  

### Initial Build
I built out this project perhaps a bit too quickly.  The directions to validate each layer weren't completely clear and I didn't know how to build and install, so I wasn't able to validate.  I got the pieces built, and injected the directions to validate input along the way.  It went fine, with one error that I noted, but considered possibly useful.  That was that the list displayed all _non-closed_ items rather than **just open** issues.  As a result when I started testing, I noticed the issue.  I have made a distinction between `done` and `closed` items in my schema. So the list will show `done` items in the default list without sorting.  

Once I was finished creating the tracker, I figured out how to install it and ran it through some testing.  I found a few sticking points and started putting issues into the tracker for later recall.  This is when I noticed a few more issues, and added them to the tracker too.  That gives me some usability and feature issues to tackle, and now I'm ready to run my first review.

## Issues and Review
At this point, I'll be storing the issues in the tracker for the duration of the build and documenting thought process here, primarily.  I'll fill in my experience, and keep the issues in that location.  
