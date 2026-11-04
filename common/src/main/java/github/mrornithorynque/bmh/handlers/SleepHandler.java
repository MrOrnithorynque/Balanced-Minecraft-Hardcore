package github.mrornithorynque.bmh.handlers;

import github.mrornithorynque.bmh.utilities.BMHGameRules;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.entity.player.Player;

import net.minecraft.world.food.FoodData;
import net.minecraft.world.effect.MobEffectInstance;
import net.minecraft.world.effect.MobEffects;

import org.slf4j.Logger;

import com.mojang.logging.LogUtils;

public class SleepHandler {

    private static final Logger LOGGER = LogUtils.getLogger();

    // Called when a player gets out of bed (Forge: PlayerWakeUpEvent).
    public static void onPlayerWakeUp(Player sleeper) {

        if (sleeper instanceof ServerPlayer) {

            ServerPlayer player = (ServerPlayer) sleeper;
            ServerLevel serverLevel = player.serverLevel();

            if (serverLevel.getGameRules().getBoolean(BMHGameRules.RULE_FOOD_AND_HEALTH_REGEN_ON_WAKE_UP)) {

                healthAndFoodRegen(player);
            }
        }
    }

    static void healthAndFoodRegen(ServerPlayer player) {

        FoodData foodData = player.getFoodData();

        float playerHealth = player.getHealth();
        float amountOfExhaustion = 4f;
        int foodLevel = foodData.getFoodLevel();

        if (foodLevel >= 15 /* 75% */) {

            player.heal(6);
            foodData.addExhaustion(amountOfExhaustion);

            if (playerHealth >= 20f /* 100% */) {

                MobEffectInstance digSpeed = new MobEffectInstance(MobEffects.DIG_SPEED, 3600, 1, false, false, true);
                MobEffectInstance resistance = new MobEffectInstance(MobEffects.DAMAGE_RESISTANCE, 3600, 1, false, false, true);

                player.addEffect(digSpeed);
                player.addEffect(resistance);
            }

            foodData.setFoodLevel(foodLevel - 4);

        } else if (foodLevel >= 10 /* 50% */) {

            player.heal(4);
            foodData.addExhaustion(amountOfExhaustion);
            foodData.setFoodLevel(foodLevel - 2);

        } else if (foodLevel >= 5 /* 25% */) {

            player.heal(3);
            foodData.addExhaustion(amountOfExhaustion);
            foodData.setFoodLevel(foodLevel - 1);

        } else if (foodLevel > 1 /* 5% */) {

            player.heal(1);
            foodData.addExhaustion(amountOfExhaustion / 4);
            foodData.setFoodLevel(foodLevel - 1);
        }
    }
}