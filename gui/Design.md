## GUI

To allow use to complete with other solutions we really need a GUI

Proposed frontend frameworks:

-   React

Proposed GUI capabilities

-   Handle login flow
-   display the images the user has access to and various info/stats like annotations
-   support various admin tasks such as deleting images or giving users access to repos etc
-   display the results of security scans etc
-   display the history of tags and images
-   allowing searching of images

### Proposed Initial GUI capabilities

Handle login flow
display the images the user has access to and various info/stats like annotations

# Architecture

The frontend will interact with the existing trow client interface api using a provided TROW_REGISTRY_URL endpoint supplied to allow flexibility during deployment.

### Architecture Notes
