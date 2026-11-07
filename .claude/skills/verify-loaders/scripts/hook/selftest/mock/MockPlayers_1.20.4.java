package github.mrornithorynque.bmh.selftest;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.TreeSet;
import java.util.UUID;
import java.util.stream.Collectors;

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
import net.minecraft.nbt.CompoundTag;
import net.minecraft.nbt.ListTag;
import net.minecraft.world.item.enchantment.EnchantmentHelper;
import net.minecraft.world.item.enchantment.Enchantments;
import net.minecraft.network.ConnectionProtocol;
import net.minecraft.server.network.CommonListenerCookie;
import net.minecraft.core.BlockPos;
import net.minecraft.world.inventory.AbstractContainerMenu;
import net.minecraft.world.inventory.ClickType;
import net.minecraft.world.effect.MobEffectInstance;
import net.minecraft.world.effect.MobEffects;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.EntityType;
import net.minecraft.world.entity.LivingEntity;
import net.minecraft.world.entity.monster.Husk;
import net.minecraft.world.entity.animal.IronGolem;
import net.minecraft.world.level.GameRules;

/**
 * THROWAWAY (see selftest.sh): a player with a connection, placed like a joining one, for Minecraft 1.20.2 to 1.20.4.
 * The same steps as vanilla's GameTestHelper.makeMockServerPlayerInLevel.
 */
public final class MockPlayers {

    // The channel of each mock player: it keeps every packet the server sends, in order. A respawn keeps the connection.
    private static final Map<UUID, EmbeddedChannel> CHANNELS = new HashMap<>();

    private MockPlayers() {
    }

    public static ServerPlayer place(MinecraftServer server, ServerLevel level, String name) {
        CommonListenerCookie cookie = CommonListenerCookie.createInitial(new GameProfile(UUID.randomUUID(), name));
        ServerPlayer player = new ServerPlayer(server, level, cookie.gameProfile(), cookie.clientInformation());
        Connection connection = new Connection(PacketFlow.SERVERBOUND);
        EmbeddedChannel channel = new EmbeddedChannel(connection);
        CHANNELS.put(player.getUUID(), channel);
        channel.attr(Connection.ATTRIBUTE_SERVERBOUND_PROTOCOL).set(ConnectionProtocol.PLAY.codec(PacketFlow.SERVERBOUND));
        server.getPlayerList().placeNewPlayer(connection, player, cookie);
        return player;
    }

    public static ServerPlayer respawn(MinecraftServer server, ServerPlayer player) {
        return server.getPlayerList().respawn(player, false);
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
        return packet instanceof ClientboundSetTitleTextPacket title ? title.getText() : null;
    }

    public static void enchantSharpness(ServerLevel level, ItemStack stack, int value) {
        stack.enchant(Enchantments.SHARPNESS, value);
    }

    public static int sharpness(ServerLevel level, ItemStack stack) {
        return EnchantmentHelper.getItemEnchantmentLevel(Enchantments.SHARPNESS, stack);
    }

    /** Puts one stack in a bundle the way the Eternal Bundle stores it (an "Items" list in the tag). */
    public static void fillBundle(ItemStack bundle, ItemStack content) {
        ListTag items = new ListTag();
        items.add(content.save(new CompoundTag()));
        bundle.getOrCreateTag().put("Items", items);
    }

    public static boolean sameData(ItemStack a, ItemStack b) {
        return ItemStack.isSameItemSameTags(a, b);
    }

    /** Kills a mock player. */
    public static void kill(ServerPlayer player) {
        player.kill();
    }

    /** Selects a hotbar slot. */
    public static void select(ServerPlayer player, int slot) {
        player.getInventory().selected = slot;
    }

    /** Puts a stack in the off hand without any equip side effect. */
    public static void setOffhand(ServerPlayer player, ItemStack stack) {
        player.getInventory().offhand.set(0, stack);
    }

    /** An iron golem without AI, added to the level: a target with 100 health that hits back at no one. */
    public static LivingEntity golem(ServerLevel level, double x, double y, double z) {
        IronGolem golem = EntityType.IRON_GOLEM.create(level);
        golem.moveTo(x, y, z, 0, 0);
        golem.setNoAi(true);
        level.addFreshEntity(golem);
        return golem;
    }

    /** A husk without AI, added to the level: an Enemy that the Eternal Sword's echo can pick, and that hits back at no one. */
    public static LivingEntity husk(ServerLevel level, double x, double y, double z) {
        Husk husk = EntityType.HUSK.create(level);
        husk.moveTo(x, y, z, 0, 0);
        husk.setNoAi(true);
        level.addFreshEntity(husk);
        return husk;
    }

    /** The world spawn point. */
    public static BlockPos worldSpawn(ServerLevel level) {
        return level.getSharedSpawnPos();
    }

    // A game rule key is a GameRules.Key up to 1.21.1 and a GameRule on 26.1: the tests hold it as an Object.

    @SuppressWarnings("unchecked")
    public static boolean getRule(MinecraftServer server, Object key) {
        return server.getGameRules().getRule((GameRules.Key<GameRules.BooleanValue>) key).get();
    }

    @SuppressWarnings("unchecked")
    public static void setRule(MinecraftServer server, Object key, boolean value) {
        server.getGameRules().getRule((GameRules.Key<GameRules.BooleanValue>) key).set(value, server);
    }

    /** The key of vanilla's reducedDebugInfo rule. */
    public static Object reducedDebugInfoRule() {
        return GameRules.RULE_REDUCEDDEBUGINFO;
    }

    /** The paths of the recipes in the bmh namespace, sorted. */
    public static Set<String> bmhRecipes(MinecraftServer server) {
        return server.getRecipeManager().getRecipeIds()
                .filter(id -> id.getNamespace().equals("bmh"))
                .map(id -> id.getPath())
                .collect(Collectors.toCollection(TreeSet::new));
    }

    /** The Haste effect of a player, or null. */
    public static MobEffectInstance haste(ServerPlayer player) {
        return player.getEffect(MobEffects.DIG_SPEED);
    }

    /** The Resistance effect of a player, or null. */
    public static MobEffectInstance resistance(ServerPlayer player) {
        return player.getEffect(MobEffects.DAMAGE_RESISTANCE);
    }

    /** Shift-clicks a slot of a menu like a player: the result of a smithing table or an anvil goes to the inventory. */
    public static void quickMove(AbstractContainerMenu menu, int slot, ServerPlayer player) {
        menu.clicked(slot, 0, ClickType.QUICK_MOVE, player);
    }

    /** True when the entity has this command tag. */
    public static boolean hasTag(Entity entity, String tag) {
        return entity.getTags().contains(tag);
    }

    /**
     * ServerPlayer.spawnInvulnerableTime, private on this version: read by reflection (Mojang names in dev). BMH sets it
     * to 200 after a distant respawn, vanilla to 60.
     */
    public static int immunityTicks(ServerPlayer player) {
        try {
            java.lang.reflect.Field field = ServerPlayer.class.getDeclaredField("spawnInvulnerableTime");
            field.setAccessible(true);
            return field.getInt(player);
        } catch (ReflectiveOperationException e) {
            throw new IllegalStateException(e);
        }
    }
}
