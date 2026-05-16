import org.json.JSONObject;

import com.azure.core.credential.TokenCredential;
import com.azure.identity.ClientSecretCredentialBuilder;
import com.microsoft.graph.serviceclient.GraphServiceClient;

/* Created with LazyJava */
public class Project {

    public static void main(String[] args) {
        System.out.println("Hello world!");
        System.out.println("Welcome to your LazyJava project");
        JSONObject jo = new JSONObject("{ \"abc\" : \"def\" }");

        System.out.println(jo.toString());
        // Using Azure Identity for authentication
        TokenCredential credential = new ClientSecretCredentialBuilder()
                .clientId("YOUR_CLIENT_ID")
                .tenantId("YOUR_TENANT_ID")
                .clientSecret("YOUR_CLIENT_SECRET")
                .build();

        GraphServiceClient graphClient = new GraphServiceClient(credential);

    }
}
