#!/bin/bash

echo "Testing macOS Contacts access..."
echo ""

osascript -e 'tell application "Contacts"
set outLines to {}
repeat with p in every person
try
set personName to (name of p) as text
set personEmail to ""
if ((count of emails of p) > 0) then set personEmail to (value of first email of p) as text
set personPhone to ""
if ((count of phones of p) > 0) then set personPhone to (value of first phone of p) as text
set end of outLines to personName & tab & personEmail & tab & personPhone
if ((count of outLines) >= 10) then exit repeat
end try
end repeat
set AppleScript'"'"'s text item delimiters to linefeed
return outLines as text
end tell'

echo ""
echo "If you see contacts above, the integration is working!"
echo "If you see an error, you may need to grant Contacts permission in System Settings > Privacy & Security > Contacts"
