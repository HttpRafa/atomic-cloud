package io.atomic.cloud.api.manage;

import io.atomic.cloud.api.resource.Resource;
import io.atomic.cloud.api.resource.complex.Group;
import io.atomic.cloud.api.resource.complex.Node;
import io.atomic.cloud.api.resource.complex.Server;
import io.atomic.cloud.api.resource.simple.SimpleGroup;
import io.atomic.cloud.api.resource.simple.SimpleNode;
import io.atomic.cloud.api.resource.simple.SimpleServer;
import io.atomic.cloud.api.transfer.Transfers;
import java.util.concurrent.CompletableFuture;

public interface Privileged {

    /**
     * This will return the transfer object in privileged mode
     * @return The transfer object
     */
    Transfers transfers();

    /**
     * This will request the controller to stop
     * After calling this method, this server will be killed or stopped
     * @return A future that completes when the controller handled the request
     */
    CompletableFuture<Void> stopController();

    /**
     * Sets the status of a cloud resource
     * @param resource The resource to set group or node
     * @param active If the resource should be set to active or inactive
     * @return A future that completes when the controller handled the request
     */
    CompletableFuture<Void> setResource(Resource resource, boolean active);

    /**
     * This will try to delete the provided resource from the controller
     * @param resource The resource to delete
     * @return A future that completes when the controller handled the request
     */
    CompletableFuture<Void> deleteResource(Resource resource);

    /**
     * Creates a node on the controller
     * @param node Node to create
     * @return A future that completes when the controller handled the request
     */
    CompletableFuture<Void> createNode(Node node);

    /**
     * Creates a group on the controller
     * @param group Group to create
     * @return A future that completes when the controller handled the request
     */
    CompletableFuture<Void> createGroup(Group group);

    /**
     * Writes data to the screen of a server
     * NOTE: The plugin driving the node must support this
     * @param server The server to write to
     * @param data The data to write
     * @return A future that completes when the controller handled the request
     */
    CompletableFuture<Void> writeToScreen(SimpleServer server, byte[] data);

    /**
     * Resolves the node to a complex node
     * @param node The node to resolve
     * @return A future that completes when the controller handled the request
     */
    CompletableFuture<Node> node(SimpleNode node);

    /**
     * Resolves the group to a complex group
     * @param group The group to resolve
     * @return A future that completes when the controller handled the request
     */
    CompletableFuture<Group> group(SimpleGroup group);

    /**
     * Resolves the server to a complex server
     * @param server The server to resolve
     * @return A future that completes when the controller handled the request
     */
    CompletableFuture<Server> server(SimpleServer server);
}
