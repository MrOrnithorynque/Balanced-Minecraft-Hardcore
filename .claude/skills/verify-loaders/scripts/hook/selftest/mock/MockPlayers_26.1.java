package github.mrornithorynque.bmh.selftest;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.TreeSet;
import java.util.UUID;

import com.mojang.authlib.GameProfile;

import io.netty.channel.embedded.EmbeddedChannel;
import net.minecraft.network.Connection;
import net.minecraft.network.protocol.Packet;
import net.minecraft.network.protocol.PacketFlow;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.network.chat.Component;
import net.minecraft.network.protocol.game.ClientboundSetTitleTextPacket;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.ItemStackTemplate;
import net.minecraft.core.Holder;
import net.minecraft.core.component.DataComponents;
import net.minecraft.core.registries.Registries;
import net.minecraft.world.item.component.BundleContents;
import net.minecraft.world.item.enchantment.Enchantment;
import net.minecraft.world.item.enchantment.EnchantmentHelper;
import net.minecraft.world.item.enchantment.Enchantments;
import net.minecraft.server.network.CommonListenerCookie;
import net.minecraft.world.entity.Entity;
import net.minecraft.network.protocol.game.ServerboundClientCommandPacket;
import net.minecraft.network.protocol.game.ServerboundPlayerLoadedPacket;
import net.minecraft.server.network.ServerGamePacketListenerImpl;
import net.minecraft.core.BlockPos;
import net.minecraft.resources.Identifier;
import net.minecraft.world.inventory.AbstractContainerMenu;
import net.minecraft.world.inventory.ContainerInput;
import net.minecraft.world.effect.MobEffectInstance;
import net.minecraft.world.effect.MobEffects;
import net.minecraft.world.entity.EntitySpawnReason;
import net.minecraft.world.entity.EntityType;
import net.minecraft.world.entity.LivingEntity;
import net.minecraft.world.entity.monster.Husk;
import net.minecraft.world.entity.animal.golem.IronGolem;
import net.minecraft.world.entity.player.Inventory;
import net.minecraft.world.item.crafting.RecipeHolder;
import net.minecraft.world.level.gamerules.GameRule;
import net.minecraft.world.level.gamerules.GameRules;

/**
 * THROWAWAY (see selftest.sh): a player with a connection, placed like a joining one, for Minecraft 26.1 (built on 26.1.2).
 * The same steps as vanilla's GameTestHelper.makeMockServerPlayerInLevel.
 */
public final class MockPlayers {

    // The channel of each mock player: it keeps every packet the server sends, in order. A respawn keeps the connection.
    private static final Map<UUID, EmbeddedChannel> CHANNELS = new HashMap<>();

    private MockPlayers() {
    }

    public static ServerPlayer place(MinecraftServer server, ServerLevel level, String name) {
        CommonListenerCookie cookie = CommonListenerCookie.createInitial(new GameProfile(UUID.randomUUID(), name), false);
        ServerPlayer player = new ServerPlayer(server, level, cookie.gameProfile(), cookie.clientInformation());
        Connection connection = new Connection(PacketFlow.SERVERBOUND);
        EmbeddedChannel channel = new EmbeddedChannel(connection);
        CHANNELS.put(player.getUUID(), channel);
        server.getPlayerList().placeNewPlayer(connection, player, cookie);
        clientLoaded(player);
        return player;
    }

    /**
     * Since 26.1 a player takes no damage, not even /kill, until its client says that it has loaded (hasClientLoaded):
     * 60 ticks after a join, and after a death until the Respawn command. A real client sends ServerboundPlayerLoadedPacket
     * then. The mock does the same, or it could not die.
     */
    private static void clientLoaded(ServerPlayer player) {
        player.connection.handleAcceptPlayerLoad(new ServerboundPlayerLoadedPacket());
    }

    /** The Respawn button: the same packet that a client sends, so the connection also forgets that it waits for the respawn. */
    public static ServerPlayer respawn(MinecraftServer server, ServerPlayer player) {
        ServerGamePacketListenerImpl connection = player.connection;
        connection.handleClientCommand(new ServerboundClientCommandPacket(ServerboundClientCommandPacket.Action.PERFORM_RESPAWN));
        ServerPlayer back = connection.getPlayer();
        clientLoaded(back);
        return back;
    }

    /** Every packet sent to this player since the last call, oldest first. */
    public static List<Packet<?>> sent(ServerPlayer player) {
        List<Packet<?>> packets = new ArrayList<>();
        EmbeddedChannel channel = CHANNELS.get(player.getUUID());
        if (channel == null) return packets;
        for (Object message = channel.readOutbound(); message != null; message = channel.readOutbound()) {
            if (message instanceof Packet<?> packet) packets.add(packet);
        }
        return packets;
    }

    // Version helpers: the calls the tests make that changed between Minecraft versions. Every copy has the same ones.

    /** The text of a title packet, or null for any other packet. */
    public static Component titleText(Packet<?> packet) {
        return packet instanceof ClientboundSetTitleTextPacket title ? title.text() : null;
    }

    private static Holder<Enchantment> sharpnessHolder(ServerLevel level) {
        return level.registryAccess().lookupOrThrow(Registries.ENCHANTMENT).getOrThrow(Enchantments.SHARPNESS);
    }

    public static void enchantSharpness(ServerLevel level, ItemStack stack, int value) {
        stack.enchant(sharpnessHolder(level), value);
    }

    public static int sharpness(ServerLevel level, ItemStack stack) {
        return EnchantmentHelper.getItemEnchantmentLevel(sharpnessHolder(level), stack);
    }

    /** Puts one stack in a bundle the way the Eternal Bundle stores it (the BUNDLE_CONTENTS component). */
    public static void fillBundle(ItemStack bundle, ItemStack content) {
        bundle.set(DataComponents.BUNDLE_CONTENTS, new BundleContents(List.of(ItemStackTemplate.fromNonEmptyStack(content))));
    }

    public static boolean sameData(ItemStack a, ItemStack b) {
        return ItemStack.isSameItemSameComponents(a, b);
    }

    /** Kills a mock player. */
    public static void kill(ServerPlayer player) {
        player.kill(player.level());
    }

    /** Selects a hotbar slot. */
    public static void select(ServerPlayer player, int slot) {
        player.getInventory().setSelectedSlot(slot);
    }

    /** Puts a stack in the off hand without any equip side effect. */
    public static void setOffhand(ServerPlayer player, ItemStack stack) {
        player.getInventory().setItem(Inventory.SLOT_OFFHAND, stack);
    }

    /** An iron golem without AI, added to the level: a target with 100 health that hits back at no one. */
    public static LivingEntity golem(ServerLevel level, double x, double y, double z) {
        IronGolem golem = EntityType.IRON_GOLEM.create(level, EntitySpawnReason.COMMAND);
        golem.snapTo(x, y, z, 0, 0);
        golem.setNoAi(true);
        level.addFreshEntity(golem);
        return golem;
    }

    /** A husk without AI, added to the level: an Enemy that the Eternal Sword's echo can pick, and that hits back at no one. */
    public static LivingEntity husk(ServerLevel level, double x, double y, double z) {
        Husk husk = EntityType.HUSK.create(level, EntitySpawnReason.COMMAND);
        husk.snapTo(x, y, z, 0, 0);
        husk.setNoAi(true);
        level.addFreshEntity(husk);
        return husk;
    }

    /** The world spawn point. */
    public static BlockPos worldSpawn(ServerLevel level) {
        return level.getRespawnData().pos();
    }

    // A game rule key is a GameRules.Key up to 1.21.1 and a GameRule on 26.1: the tests hold it as an Object.

    @SuppressWarnings("unchecked")
    public static boolean getRule(MinecraftServer server, Object key) {
        return server.getGameRules().get((GameRule<Boolean>) key);
    }

    @SuppressWarnings("unchecked")
    public static void setRule(MinecraftServer server, Object key, boolean value) {
        server.getGameRules().set((GameRule<Boolean>) key, value, server);
    }

    /** The key of vanilla's reducedDebugInfo rule. */
    public static Object reducedDebugInfoRule() {
        return GameRules.REDUCED_DEBUG_INFO;
    }

    /** The paths of the recipes in the bmh namespace, sorted. */
    public static Set<String> bmhRecipes(MinecraftServer server) {
        Set<String> paths = new TreeSet<>();
        for (RecipeHolder<?> holder : server.getRecipeManager().getRecipes()) {
            Identifier id = holder.id().identifier();
            if (id.getNamespace().equals("bmh")) paths.add(id.getPath());
        }
        return paths;
    }

    /** The Haste effect of a player, or null. */
    public static MobEffectInstance haste(ServerPlayer player) {
        return player.getEffect(MobEffects.HASTE);
    }

    /** The Resistance effect of a player, or null. */
    public static MobEffectInstance resistance(ServerPlayer player) {
        return player.getEffect(MobEffects.RESISTANCE);
    }

    /** Shift-clicks a slot of a menu like a player: the result of a smithing table or an anvil goes to the inventory. */
    public static void quickMove(AbstractContainerMenu menu, int slot, ServerPlayer player) {
        menu.clicked(slot, 0, ContainerInput.QUICK_MOVE, player);
    }

    /** True when the entity has this command tag. */
    public static boolean hasTag(Entity entity, String tag) {
        return entity.entityTags().contains(tag);
    }

    /**
     * The ticks of damage immunity a player has left after a respawn. BMH sets 200 after a distant respawn. 26.1 removed
     * vanilla's counter (ServerPlayer.spawnInvulnerableTime), so this reads the private counter that BMH's
     * F_ServerPlayerJourneyMixin adds: 0 when no journey shield is on, and vanilla's 60 does not show here.
     */
    public static int immunityTicks(ServerPlayer player) {
        try {
            java.lang.reflect.Field field = ServerPlayer.class.getDeclaredField("bmh$journeyTicks");
            field.setAccessible(true);
            return field.getInt(player);
        } catch (ReflectiveOperationException e) {
            throw new IllegalStateException(e);
        }
    }
}
