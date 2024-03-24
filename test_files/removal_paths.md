# List of REDACTED paths by Removal

`json
$.entities[?(@.roles[0]=='registrant')].vcardArray[1][?(@[0]=='org')] -- item in vcardarray, how would we display?
$.entities[?(@.roles[0]=='registrant')].vcardArray[1][?(@[0]=='email')] -- item in vcardarray, how would we display?
$.entities[?(@.roles[0]=='registrant')].vcardArray[1][?(@[1].type=='voice')] -- item in vcardarray, how would we display?
$.entities[?(@.roles[0]=='technical')].vcardArray[1][?(@[0]=='email')] -- item in vcardarray, how would we display?
$.entities[?(@.roles[0]=='technical')].vcardArray[1][?(@[1].type=='voice')] -- item in vcardarray, how would we display?
$.entities[?(@.roles[0]=='technical')].vcardArray[1][?(@[1].type=='fax')] -- item in vcardarray, how would we display?
$.entities[?(@.roles[0]=='administrative')]  -- this is a whole entity
$.entities[?(@.roles[0]=='billing')] -- this is a whole entity
`

# Problems
for this path `$.entities[?(@.roles[0]=='billing')]` the whole thing has been removed. How and where would we put it back?
We cant handle `..`  meaning recursive paths, for example how would we handle `$..[?(@.roles[*]roles == 'technical]`
We can't recreate stuff like that
We can't make the whole thing above as redacted b/c we are only marking strings or numbers as redacted with **
if part of an array (for example the 3rd item) has been removed what are we supposed to do, mark each item in the array as redacted?
And If I only chop off the end, what about the path here: `$.entities[?(@.roles[0]=='billing')]` .. that gets me `$.entities` I can't mark them as redacted
