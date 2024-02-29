Have a workspace with multiple members.

Comment out the members in the workspace Nargo.toml file, except for one member.

Then compile that member using `nargo compile --package {member}

You will get warnings about the other members having unused members