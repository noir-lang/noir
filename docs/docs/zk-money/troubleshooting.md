# Troubleshooting Guide

## Hung on "Connecting to Rollup Provider" on sign-up or log-in

You may be presented with an "AbortError: Transaction Aborted" when attempted to sign up or log-in after a system update.

In that case, navigate to your browser console (F12 or Alt-Cmd-J) and the Application tab on the top bar. Once there, click "Storage" under the "Application" group and click "Clear site data." Refresh the page.

![image](https://user-images.githubusercontent.com/15220860/177643292-e39ce717-8a58-4916-ad51-74e10c7685d4.png)

### Firefox

If you are using Firefox, open the browser console (F12 or right click and select Inspect). Go to the Storage tab on the top bar. Select the Indexed DB tab, select https://zk.money and right click "hummus" and delete it. Refresh the page.

![firefox dev tools](https://user-images.githubusercontent.com/18372439/178279060-8c8b6d58-f0ae-4986-9649-390deaa611cb.png)

## Frequently Asked Questions

Check the [frequently asked questions page](/how-aztec-works/faq) to see if your problem is addressed there. 

If it isn't, join our [Discord server](https://discord.com/invite/UDtJr9u) and get an answer from the community.
