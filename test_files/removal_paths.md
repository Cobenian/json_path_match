# List of REDACTED paths by Removal

`json
$.entities[?(@.roles[0]=='registrant')].vcardArray[1][?(@[0]=='org')]
$.entities[?(@.roles[0]=='registrant')].vcardArray[1][?(@[0]=='email')] 
$.entities[?(@.roles[0]=='registrant')].vcardArray[1][?(@[1].type=='voice')]
$.entities[?(@.roles[0]=='technical')].vcardArray[1][?(@[0]=='email')] 
$.entities[?(@.roles[0]=='technical')].vcardArray[1][?(@[1].type=='voice')] 
$.entities[?(@.roles[0]=='technical')].vcardArray[1][?(@[1].type=='fax')] 
$.entities[?(@.roles[0]=='administrative')] 
$.entities[?(@.roles[0]=='billing')] 
`

# Problems
for this path `$.entities[?(@.roles[0]=='billing')]` the whole thing has been removed. How and where would we put it back?
We cant handle `..`  meaning recursive paths, for example how would we handle `$..[?(@.roles[*]roles == 'technical]`
We can't recreate stuff like that
We can't make the whole thing above as redacted b/c we are only marking strings or numbers as redacted with **
if part of an array (for example the 3rd item) has been removed what are we supposed to do, mark each item in the array as redacted?
And If I only chop off the end, what about the path here: `$.entities[?(@.roles[0]=='billing')]` .. that gets me `$.entities` I can't mark them as redacted


$.entities[?(@.roles[0]=='registrant')].vcardArray[1][?(@[0]=='org')]

1. $.entities
2. [?(@.roles[0]=='registrant')]
3. .vcardArray
4. [1]
5. [?(@[0]=='org')]