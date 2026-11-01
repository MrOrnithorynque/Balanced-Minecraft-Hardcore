package github.mrornithorynque.bmh.platform;

import java.util.ServiceLoader;

public class Services {

    public static final IPlatformHelper PLATFORM = ServiceLoader.load(IPlatformHelper.class)
            .findFirst()
            .orElseThrow(() -> new IllegalStateException("No IPlatformHelper found: is the loader module missing?"));
}
