package github.mrornithorynque.bmh.fabric;

import com.terraformersmc.modmenu.api.ConfigScreenFactory;
import com.terraformersmc.modmenu.api.ModMenuApi;

import github.mrornithorynque.bmh.client.config.ConfigScreen;

/**
 * The "modmenu" entrypoint: the Config button of BMH in Mod Menu. Only Mod Menu loads this class, so
 * BMH does not need Mod Menu to run (it is compile only, see fabric/build.gradle). Client only.
 */
public class ModMenuIntegration implements ModMenuApi {

    @Override
    public ConfigScreenFactory<?> getModConfigScreenFactory() {

        return ConfigScreen::new;
    }
}
