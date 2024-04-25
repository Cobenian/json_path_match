# Return Results for check_json_paths

The following are codes returned from a function called `check_json_paths`
This is the part that reads a list of redaction paths (json paths) and attempts to find what is there.

The * indicates the ones I have found so far with the testing I've done and the small demo I gave.
Each one lines up with one of the Redaction Methods.
It is certainly conceivable that some of the other types could show up, especially if they claimed a redaction but didn't actually remove it.
That is why we check for all the below.

- Removed1 (*) paths array is empty, finder.find_as_path() found nothing `Redaction by Removal`
- Removed2 value in paths array is null (have never found this)
- Removed3 fall through, value in paths array is not anything else (have never found this)
- Removed4 what we found was not a JSON::Value::string (have never found this)
- Removed5 what finder.find_as_path() returned was not a Value::Array (have never found this, could possibly be an error)
- Empty1 (*) what we found in the value paths array was a string but has no value (yes, this is a little weird, but does exist) `Redaction by Empty Value`
- Empty2 (*) what we found in the value paths array was a string but it is an empty string `Redaction by Empty Value`
- Replaced1 (*) what we found in the value paths array was a string and it does have a value `Redaction by Partial Value` and/or `Redaction by Replacement Value`
- Replaced2 what we found in the value paths array was _another_ array (have never found this)
- Replaced3 what we found in the value paths array was an object (have never found this)

- Error - we do check for Errors, at the moment we println and should blow up so I can see what happened